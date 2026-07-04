#!/usr/bin/env bash
set -euo pipefail

script_dir="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "$script_dir/.." && pwd)"
build_dir="$(mktemp -d "${TMPDIR:-/tmp}/madobe-direct-capture-preflight.XXXXXX")"
trap 'rm -rf "$build_dir"' EXIT

require_tool() {
  local tool="$1"
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'required tool missing: %s; run inside nix develop\n' "$tool" >&2
    exit 127
  }
}

require_pkg_config() {
  local module="$1"
  pkg-config --exists "$module" || {
    printf 'required pkg-config module missing: %s\n' "$module" >&2
    exit 127
  }
}

require_tool cc
require_tool pkg-config
require_tool wayland-scanner

for module in wayland-client wayland-protocols gbm libdrm; do
  require_pkg_config "$module"
done

protocol_dir="$(pkg-config --variable=pkgdatadir wayland-protocols)"
foreign_toplevel_xml="$protocol_dir/staging/ext-foreign-toplevel-list/ext-foreign-toplevel-list-v1.xml"
ext_source_xml="$protocol_dir/staging/ext-image-capture-source/ext-image-capture-source-v1.xml"
ext_capture_xml="$protocol_dir/staging/ext-image-copy-capture/ext-image-copy-capture-v1.xml"
dmabuf_xml="$protocol_dir/stable/linux-dmabuf/linux-dmabuf-v1.xml"

for protocol_xml in "$foreign_toplevel_xml" "$ext_source_xml" "$ext_capture_xml" "$dmabuf_xml"; do
  if [ ! -f "$protocol_xml" ]; then
    printf 'required Wayland protocol XML missing: %s\n' "$protocol_xml" >&2
    exit 127
  fi
done

wayland-scanner client-header \
  "$foreign_toplevel_xml" \
  "$build_dir/ext-foreign-toplevel-list-v1-client-protocol.h"
wayland-scanner private-code \
  "$foreign_toplevel_xml" \
  "$build_dir/ext-foreign-toplevel-list-v1-protocol.c"
wayland-scanner client-header \
  "$ext_source_xml" \
  "$build_dir/ext-image-capture-source-v1-client-protocol.h"
wayland-scanner private-code \
  "$ext_source_xml" \
  "$build_dir/ext-image-capture-source-v1-protocol.c"
wayland-scanner client-header \
  "$ext_capture_xml" \
  "$build_dir/ext-image-copy-capture-v1-client-protocol.h"
wayland-scanner private-code \
  "$ext_capture_xml" \
  "$build_dir/ext-image-copy-capture-v1-protocol.c"
wayland-scanner client-header \
  "$dmabuf_xml" \
  "$build_dir/linux-dmabuf-v1-client-protocol.h"
wayland-scanner private-code \
  "$dmabuf_xml" \
  "$build_dir/linux-dmabuf-v1-protocol.c"

IFS=' ' read -r -a pkg_config_flags <<<"$(pkg-config --cflags --libs wayland-client gbm libdrm)"
cc -std=c11 -Wall -Wextra -Werror \
  -I"$build_dir" \
  "$repo_root/crates/capture/tools/direct_capture_preflight.c" \
  "$build_dir/ext-foreign-toplevel-list-v1-protocol.c" \
  "$build_dir/ext-image-capture-source-v1-protocol.c" \
  "$build_dir/ext-image-copy-capture-v1-protocol.c" \
  "$build_dir/linux-dmabuf-v1-protocol.c" \
  "${pkg_config_flags[@]}" \
  -o "$build_dir/direct-capture-preflight"

"$build_dir/direct-capture-preflight"
printf 'wayland-client %s\n' "$(pkg-config --modversion wayland-client)"
printf 'wayland-protocols %s\n' "$(pkg-config --modversion wayland-protocols)"
printf 'gbm %s\n' "$(pkg-config --modversion gbm)"
printf 'libdrm %s\n' "$(pkg-config --modversion libdrm)"
printf 'generated %s\n' "$foreign_toplevel_xml"
printf 'generated %s\n' "$ext_source_xml"
printf 'generated %s\n' "$ext_capture_xml"
printf 'generated %s\n' "$dmabuf_xml"
