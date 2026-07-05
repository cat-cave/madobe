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

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::io::Cursor;

    use madobe_protocol::{
        EncodedVideoFrameMetadata, PayloadHash, PayloadHashAlgorithm, VideoCodec,
    };

    use super::{FIXED_HEADER_LEN, read_sample, write_sample};
    use crate::VideoSample;
    use crate::video_smoke::SmokeErrorKind;

    const HASH_HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    const PAYLOAD: &[u8] = b"wire-contract";

    #[test]
    fn write_sample_emits_stable_header_and_round_trips() -> Result<(), Box<dyn Error>> {
        let sample = sample(HASH_HEX)?;
        let mut wire_bytes = Vec::new();

        write_sample(&mut wire_bytes, &sample)?;

        assert_eq!(wire_bytes.len(), FIXED_HEADER_LEN + PAYLOAD.len());
        assert_eq!(&wire_bytes[0..8], b"MDBSMK01");
        assert_eq!(&wire_bytes[8..10], &1_u16.to_be_bytes());
        assert_eq!(&wire_bytes[10..18], &7_u64.to_be_bytes());
        assert_eq!(wire_bytes[18], 1);
        assert_eq!(&wire_bytes[19..23], &640_u32.to_be_bytes());
        assert_eq!(&wire_bytes[23..27], &360_u32.to_be_bytes());
        assert_eq!(
            &wire_bytes[27..35],
            &1_720_000_000_000_000_111_u64.to_be_bytes()
        );
        assert_eq!(
            &wire_bytes[35..43],
            &1_720_000_000_001_000_222_u64.to_be_bytes()
        );
        assert_eq!(wire_bytes[43], 1);
        assert_eq!(
            &wire_bytes[44..48],
            &u32::try_from(PAYLOAD.len())?.to_be_bytes()
        );
        assert_eq!(wire_bytes[48], 1);
        assert_eq!(
            &wire_bytes[49..81],
            &[
                0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
                0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67,
                0x89, 0xab, 0xcd, 0xef,
            ]
        );
        assert_eq!(&wire_bytes[FIXED_HEADER_LEN..], PAYLOAD);

        let decoded = read_sample(&mut Cursor::new(wire_bytes))?;
        assert_eq!(decoded, sample);

        Ok(())
    }

    #[test]
    fn write_sample_rejects_invalid_metadata_hash() -> Result<(), Box<dyn Error>> {
        assert_wire_error_contains(
            write_sample_to_vec(&sample("abc")?),
            "sha256 metadata was not 64 hex characters",
        );

        let uppercase = HASH_HEX.to_uppercase();
        assert_wire_error_contains(
            write_sample_to_vec(&sample(&uppercase)?),
            "sha256 metadata was not lowercase hex",
        );

        Ok(())
    }

    #[test]
    fn read_sample_rejects_invalid_wire_header_fields() -> Result<(), Box<dyn Error>> {
        let mut invalid_magic = encoded_sample()?;
        invalid_magic[0] = b'X';
        assert_wire_error_contains(
            read_sample_from_slice(&invalid_magic),
            "frame magic mismatch",
        );

        let mut unsupported_version = encoded_sample()?;
        unsupported_version[9] = 2;
        assert_wire_error_contains(
            read_sample_from_slice(&unsupported_version),
            "unsupported smoke wire version",
        );

        let mut unsupported_codec = encoded_sample()?;
        unsupported_codec[18] = 2;
        assert_wire_error_contains(
            read_sample_from_slice(&unsupported_codec),
            "unsupported smoke wire codec",
        );

        let mut unsupported_hash = encoded_sample()?;
        unsupported_hash[48] = 2;
        assert_wire_error_contains(
            read_sample_from_slice(&unsupported_hash),
            "unsupported smoke wire hash algorithm",
        );

        Ok(())
    }

    #[test]
    fn read_sample_rejects_truncated_header_and_payload() -> Result<(), Box<dyn Error>> {
        let wire_bytes = encoded_sample()?;

        assert_io_error_operation(
            read_sample_from_slice(&wire_bytes[..FIXED_HEADER_LEN - 1]),
            "read frame header",
        );
        assert_io_error_operation(
            read_sample_from_slice(&wire_bytes[..wire_bytes.len() - 1]),
            "read frame payload",
        );

        Ok(())
    }

    fn encoded_sample() -> Result<Vec<u8>, Box<dyn Error>> {
        write_sample_to_vec(&sample(HASH_HEX)?).map_err(Into::into)
    }

    fn write_sample_to_vec(
        sample: &VideoSample,
    ) -> Result<Vec<u8>, crate::video_smoke::SmokeError> {
        let mut wire_bytes = Vec::new();
        write_sample(&mut wire_bytes, sample)?;
        Ok(wire_bytes)
    }

    fn read_sample_from_slice(bytes: &[u8]) -> Result<VideoSample, crate::video_smoke::SmokeError> {
        read_sample(&mut Cursor::new(bytes))
    }

    fn sample(hash: &str) -> Result<VideoSample, Box<dyn Error>> {
        Ok(VideoSample::new(
            EncodedVideoFrameMetadata {
                frame_id: 7,
                codec: VideoCodec::Av1,
                width: 640,
                height: 360,
                capture_timestamp_ns: 1_720_000_000_000_000_111,
                encode_timestamp_ns: 1_720_000_000_001_000_222,
                keyframe: true,
                payload_bytes: u32::try_from(PAYLOAD.len())?,
                payload_hash: PayloadHash {
                    algorithm: PayloadHashAlgorithm::Sha256,
                    value: hash.to_owned(),
                },
            },
            PAYLOAD.to_vec(),
        )?)
    }

    fn assert_wire_error_contains(
        result: Result<impl Sized, crate::video_smoke::SmokeError>,
        expected: &str,
    ) {
        let Err(error) = result else {
            panic!("expected wire error containing {expected:?}");
        };
        let SmokeErrorKind::Wire { message } = error.kind() else {
            panic!("expected wire error containing {expected:?}, got {error:?}");
        };
        assert!(
            message.contains(expected),
            "wire error message should contain {expected:?}: {message}"
        );
    }

    fn assert_io_error_operation(
        result: Result<impl Sized, crate::video_smoke::SmokeError>,
        expected: &'static str,
    ) {
        let Err(error) = result else {
            panic!("expected I/O error for {expected}");
        };
        let SmokeErrorKind::Io { operation, .. } = error.kind() else {
            panic!("expected I/O error for {expected}, got {error:?}");
        };
        assert_eq!(*operation, expected);
    }
}
