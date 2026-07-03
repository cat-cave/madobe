#![doc = "Golden fixture tests for the captured frame metadata contract."]
#![allow(
    clippy::too_many_lines,
    reason = "The byte-for-byte golden fixture renderer intentionally mirrors the full JSON shape."
)]

use madobe_capture::{
    CaptureDamage, CaptureSize, CaptureSync, CaptureTimestamps, CapturedFrameMetadata, DamageRect,
    DmaBufFileDescriptor, DmaBufFormat, DmaBufFrameDescriptor, DmaBufModifier, DmaBufPlane,
    SyncFileDescriptor,
};

const GOLDEN_CAPTURE_JSON: &str = include_str!("../fixtures/captured-frame-dmabuf.json");
const EXPECTED_GOLDEN_CAPTURE_JSON: &str = concat!(
    "{\n",
    "  \"frameId\": 7,\n",
    "  \"size\": {\n",
    "    \"width\": 1920,\n",
    "    \"height\": 1080\n",
    "  },\n",
    "  \"dmaBuf\": {\n",
    "    \"format\": {\n",
    "      \"fourcc\": \"XR24\",\n",
    "      \"drmFormatCode\": 875713112\n",
    "    },\n",
    "    \"modifier\": {\n",
    "      \"kind\": \"drm\",\n",
    "      \"value\": 72057594037927936\n",
    "    },\n",
    "    \"fileDescriptors\": [\n",
    "      {\n",
    "        \"value\": 11\n",
    "      }\n",
    "    ],\n",
    "    \"planes\": [\n",
    "      {\n",
    "        \"fdIndex\": 0,\n",
    "        \"offsetBytes\": 0,\n",
    "        \"strideBytes\": 7680\n",
    "      }\n",
    "    ]\n",
    "  },\n",
    "  \"damage\": {\n",
    "    \"kind\": \"rects\",\n",
    "    \"rects\": [\n",
    "      {\n",
    "        \"x\": 0,\n",
    "        \"y\": 0,\n",
    "        \"width\": 1920,\n",
    "        \"height\": 72\n",
    "      },\n",
    "      {\n",
    "        \"x\": 128,\n",
    "        \"y\": 128,\n",
    "        \"width\": 640,\n",
    "        \"height\": 360\n",
    "      }\n",
    "    ]\n",
    "  },\n",
    "  \"sync\": {\n",
    "    \"kind\": \"explicit\",\n",
    "    \"fd\": 12\n",
    "  },\n",
    "  \"timestamps\": {\n",
    "    \"captureStartedNs\": 1720000000000000000,\n",
    "    \"frameAvailableNs\": 1720000000001666667\n",
    "  }\n",
    "}\n",
);

#[test]
fn golden_capture_fixture_covers_expected_metadata() {
    let frame = golden_frame_metadata();

    assert_eq!(frame.frame_id, 7);
    assert_eq!(frame.size, CaptureSize::new(1920, 1080));
    assert_eq!(frame.dma_buf.format.fourcc, "XR24");
    assert_eq!(frame.dma_buf.format.drm_format_code, Some(875_713_112));
    assert_eq!(
        frame.dma_buf.modifier,
        DmaBufModifier::Drm(72_057_594_037_927_936)
    );
    assert_eq!(frame.dma_buf.modifier.evidence_kind(), "drm");
    assert_eq!(
        frame.dma_buf.file_descriptors,
        vec![DmaBufFileDescriptor::new(11)]
    );
    assert_eq!(frame.dma_buf.planes, vec![DmaBufPlane::new(0, 0, 7680)]);
    assert_eq!(frame.damage.evidence_kind(), "rects");
    assert_eq!(frame.sync.evidence_kind(), "explicit");
    assert_eq!(
        frame.timestamps,
        CaptureTimestamps::new(1_720_000_000_000_000_000, 1_720_000_000_001_666_667)
    );
    assert_eq!(GOLDEN_CAPTURE_JSON, EXPECTED_GOLDEN_CAPTURE_JSON);
}

#[test]
fn golden_capture_fixture_round_trips_semantically() {
    let frame = golden_frame_metadata();

    assert_eq!(encode_golden_fixture(&frame), GOLDEN_CAPTURE_JSON);
}

fn golden_frame_metadata() -> CapturedFrameMetadata {
    CapturedFrameMetadata::new(
        7,
        CaptureSize::new(1920, 1080),
        DmaBufFrameDescriptor::new(
            DmaBufFormat::new("XR24".to_owned(), Some(875_713_112)),
            DmaBufModifier::Drm(72_057_594_037_927_936),
            vec![DmaBufFileDescriptor::new(11)],
            vec![DmaBufPlane::new(0, 0, 7680)],
        ),
        CaptureDamage::Rects(vec![
            DamageRect::new(0, 0, 1920, 72),
            DamageRect::new(128, 128, 640, 360),
        ]),
        CaptureSync::Explicit(SyncFileDescriptor::new(12)),
        CaptureTimestamps::new(1_720_000_000_000_000_000, 1_720_000_000_001_666_667),
    )
}

fn encode_golden_fixture(frame: &CapturedFrameMetadata) -> String {
    let frame_id = frame.frame_id;
    let width = frame.size.width;
    let height = frame.size.height;
    let fourcc = frame.dma_buf.format.fourcc.as_str();
    let Some(drm_format_code) = frame.dma_buf.format.drm_format_code else {
        panic!("golden fixture includes a DRM format code");
    };
    let modifier_kind = frame.dma_buf.modifier.evidence_kind();
    let DmaBufModifier::Drm(modifier_value) = frame.dma_buf.modifier else {
        panic!("golden fixture uses a DRM modifier");
    };
    let fd_value = frame.dma_buf.file_descriptors[0].value;
    let plane = frame.dma_buf.planes[0];
    let damage_kind = frame.damage.evidence_kind();
    let CaptureDamage::Rects(rects) = &frame.damage else {
        panic!("golden fixture uses damage rects");
    };
    let first_rect = rects[0];
    let second_rect = rects[1];
    let sync_kind = frame.sync.evidence_kind();
    let CaptureSync::Explicit(sync_fd) = frame.sync else {
        panic!("golden fixture uses explicit sync");
    };
    let capture_started_ns = frame.timestamps.capture_started_ns;
    let frame_available_ns = frame.timestamps.frame_available_ns;

    format!(
        concat!(
            "{{\n",
            "  \"frameId\": {frame_id},\n",
            "  \"size\": {{\n",
            "    \"width\": {width},\n",
            "    \"height\": {height}\n",
            "  }},\n",
            "  \"dmaBuf\": {{\n",
            "    \"format\": {{\n",
            "      \"fourcc\": \"{fourcc}\",\n",
            "      \"drmFormatCode\": {drm_format_code}\n",
            "    }},\n",
            "    \"modifier\": {{\n",
            "      \"kind\": \"{modifier_kind}\",\n",
            "      \"value\": {modifier_value}\n",
            "    }},\n",
            "    \"fileDescriptors\": [\n",
            "      {{\n",
            "        \"value\": {fd_value}\n",
            "      }}\n",
            "    ],\n",
            "    \"planes\": [\n",
            "      {{\n",
            "        \"fdIndex\": {fd_index},\n",
            "        \"offsetBytes\": {offset_bytes},\n",
            "        \"strideBytes\": {stride_bytes}\n",
            "      }}\n",
            "    ]\n",
            "  }},\n",
            "  \"damage\": {{\n",
            "    \"kind\": \"{damage_kind}\",\n",
            "    \"rects\": [\n",
            "      {{\n",
            "        \"x\": {first_x},\n",
            "        \"y\": {first_y},\n",
            "        \"width\": {first_width},\n",
            "        \"height\": {first_height}\n",
            "      }},\n",
            "      {{\n",
            "        \"x\": {second_x},\n",
            "        \"y\": {second_y},\n",
            "        \"width\": {second_width},\n",
            "        \"height\": {second_height}\n",
            "      }}\n",
            "    ]\n",
            "  }},\n",
            "  \"sync\": {{\n",
            "    \"kind\": \"{sync_kind}\",\n",
            "    \"fd\": {sync_fd}\n",
            "  }},\n",
            "  \"timestamps\": {{\n",
            "    \"captureStartedNs\": {capture_started_ns},\n",
            "    \"frameAvailableNs\": {frame_available_ns}\n",
            "  }}\n",
            "}}\n",
        ),
        frame_id = frame_id,
        width = width,
        height = height,
        fourcc = fourcc,
        drm_format_code = drm_format_code,
        modifier_kind = modifier_kind,
        modifier_value = modifier_value,
        fd_value = fd_value,
        fd_index = plane.fd_index,
        offset_bytes = plane.offset_bytes,
        stride_bytes = plane.stride_bytes,
        damage_kind = damage_kind,
        first_x = first_rect.x,
        first_y = first_rect.y,
        first_width = first_rect.width,
        first_height = first_rect.height,
        second_x = second_rect.x,
        second_y = second_rect.y,
        second_width = second_rect.width,
        second_height = second_rect.height,
        sync_kind = sync_kind,
        sync_fd = sync_fd.value,
        capture_started_ns = capture_started_ns,
        frame_available_ns = frame_available_ns
    )
}
