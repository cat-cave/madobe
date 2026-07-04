use std::io::{Read, Write};

use madobe_protocol::{EncodedVideoFrameMetadata, PayloadHash, PayloadHashAlgorithm, VideoCodec};

use crate::VideoSample;
use crate::video_smoke::SmokeError;

const MAGIC: &[u8; 8] = b"MDBSMK01";
const VERSION: u16 = 1;
const CODEC_AV1: u8 = 1;
const HASH_SHA256: u8 = 1;
const FIXED_HEADER_LEN: usize = 81;

pub(super) fn write_sample(
    writer: &mut impl Write,
    sample: &VideoSample,
) -> Result<(), SmokeError> {
    let hash = hash_bytes(&sample.metadata.payload_hash.value)?;
    let mut header = Vec::with_capacity(FIXED_HEADER_LEN);
    header.extend_from_slice(MAGIC);
    header.extend_from_slice(&VERSION.to_be_bytes());
    header.extend_from_slice(&sample.metadata.frame_id.to_be_bytes());
    header.push(codec_id(sample.metadata.codec));
    header.extend_from_slice(&sample.metadata.width.to_be_bytes());
    header.extend_from_slice(&sample.metadata.height.to_be_bytes());
    header.extend_from_slice(&sample.metadata.capture_timestamp_ns.to_be_bytes());
    header.extend_from_slice(&sample.metadata.encode_timestamp_ns.to_be_bytes());
    header.push(u8::from(sample.metadata.keyframe));
    header.extend_from_slice(&sample.metadata.payload_bytes.to_be_bytes());
    header.push(hash_id(sample.metadata.payload_hash.algorithm));
    header.extend_from_slice(&hash);

    writer
        .write_all(&header)
        .map_err(|error| SmokeError::io("write frame header", &error))?;
    writer
        .write_all(&sample.payload)
        .map_err(|error| SmokeError::io("write frame payload", &error))
}

pub(super) fn read_sample(reader: &mut impl Read) -> Result<VideoSample, SmokeError> {
    let mut header = [0_u8; FIXED_HEADER_LEN];
    reader
        .read_exact(&mut header)
        .map_err(|error| SmokeError::io("read frame header", &error))?;
    let metadata = parse_header(&header)?;
    let payload_len = usize::try_from(metadata.payload_bytes)
        .map_err(|_| SmokeError::wire("payload byte count did not fit usize"))?;
    let mut payload = vec![0_u8; payload_len];
    reader
        .read_exact(&mut payload)
        .map_err(|error| SmokeError::io("read frame payload", &error))?;

    VideoSample::new(metadata, payload).map_err(|error| SmokeError::transport(&error))
}

fn parse_header(header: &[u8; FIXED_HEADER_LEN]) -> Result<EncodedVideoFrameMetadata, SmokeError> {
    if &header[0..8] != MAGIC {
        return Err(SmokeError::wire("frame magic mismatch"));
    }
    if read_u16(header, 8) != VERSION {
        return Err(SmokeError::wire("unsupported smoke wire version"));
    }
    let codec = match header[18] {
        CODEC_AV1 => VideoCodec::Av1,
        _ => return Err(SmokeError::wire("unsupported smoke wire codec")),
    };
    let hash_algorithm = match header[48] {
        HASH_SHA256 => PayloadHashAlgorithm::Sha256,
        _ => return Err(SmokeError::wire("unsupported smoke wire hash algorithm")),
    };

    Ok(EncodedVideoFrameMetadata {
        frame_id: read_u64(header, 10),
        codec,
        width: read_u32(header, 19),
        height: read_u32(header, 23),
        capture_timestamp_ns: read_u64(header, 27),
        encode_timestamp_ns: read_u64(header, 35),
        keyframe: header[43] != 0,
        payload_bytes: read_u32(header, 44),
        payload_hash: PayloadHash {
            algorithm: hash_algorithm,
            value: hash_hex(&header[49..81]),
        },
    })
}

const fn codec_id(codec: VideoCodec) -> u8 {
    match codec {
        VideoCodec::Av1 => CODEC_AV1,
    }
}

const fn hash_id(algorithm: PayloadHashAlgorithm) -> u8 {
    match algorithm {
        PayloadHashAlgorithm::Sha256 => HASH_SHA256,
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_be_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
        bytes[offset + 4],
        bytes[offset + 5],
        bytes[offset + 6],
        bytes[offset + 7],
    ])
}

fn hash_bytes(value: &str) -> Result<[u8; 32], SmokeError> {
    if value.len() != 64 {
        return Err(SmokeError::wire(
            "sha256 metadata was not 64 hex characters",
        ));
    }

    let mut output = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        output[index] = (hex_value(pair[0])? << 4) | hex_value(pair[1])?;
    }

    Ok(output)
}

fn hash_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(64);
    for byte in bytes {
        output.push(hex_digit(byte >> 4));
        output.push(hex_digit(byte & 0x0f));
    }

    output
}

fn hex_value(value: u8) -> Result<u8, SmokeError> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => Err(SmokeError::wire("sha256 metadata was not lowercase hex")),
    }
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        _ => (b'a' + value - 10) as char,
    }
}
