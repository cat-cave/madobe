# madobe PRD

Status: research brief, July 2, 2026

madobe is an open, opinionated remote presence stack for a specific first-class target: Hyprland Linux hosts with NVIDIA encode hardware and native Apple clients. It is not a Moonlight/Sunshine compatibility project, not an X11-era remote desktop, and not a generic support tool.

The core promise is simple: a Mac connects to a Linux workstation as a dedicated remote display and device endpoint. The host creates a dedicated Hyprland headless output that matches the client display, captures only that output, keeps post-capture processing off the CPU when DMA-BUF import is available, encodes with NVENC, streams over a measured low-latency custom protocol, and accepts input/audio/clipboard/files/controllers only through explicit per-session permissions.

## Product Stance

madobe should feel effortless like Parsec on the reference environment. It is allowed to be unusable elsewhere.

- Host first: Linux 7.x+, Hyprland, PipeWire/XDPH, NVIDIA RTX 4090/5060 Ti-or-newer-class hardware is the reference platform.
- Client first: M5 Pro macOS native app is the first client; iOS/iPadOS comes after the desktop loop is excellent.
- Network first: LAN and Tailscale direct paths are the reference network. Public relay, cloud accounts, and hostile NAT traversal are out of scope until the local product is proven.
- Product semantics first: own session, permission, display, and latency semantics. Use existing protocol machinery where it has no product downside; do not inherit NVIDIA GameStream constraints.
- Display first: sessions are display/device endpoints, not game launch records.
- Adapter boundary, not adapter compromise: Hyprland is the reference compositor, but compositor control must sit behind a narrow capability-based adapter so madobe does not bake Hyprland command strings into unrelated capture, encode, protocol, or session code.
- Opinionated OSS: permissive license, public architecture decisions, explicit contribution boundaries, security disclosure process, and respectful rejection of requests that dilute the target. Upstream-first is preferred; forks are temporary tools, not identity.
- Reference-stack-led: madobe may copy proven UX conventions and diagnostics from other systems, but it does not carry protocol, hardware, distro, compositor, or old-client compatibility as a product requirement.
- Upstream-active: when low-level dependencies are missing capability or stability, madobe pushes the stack forward instead of waiting passively.

## Dependency Doctrine

madobe depends on fast-moving Linux graphics, media, and transport layers. That is intentional. If an upstream component is close but not good enough, the response order is:

1. No-downside alternative: use a different existing API/library only if it preserves the product contract and does not add long-term compatibility drag.
2. Contribute upstream: fix protocols, libraries, docs, drivers, portals, or compositor behavior where the change belongs generally.
3. Fork narrowly: carry a small, well-documented fork when upstream review cadence or project direction blocks the product.
4. Home-roll: build a madobe-specific component when the required behavior is product-defining and no upstream shape fits.
5. Diagnostic escape hatch: use non-reference paths only to measure, debug, or compare. They are not product modes unless explicitly promoted later with a new decision record.

This makes bleeding-edge enablement in scope. Protocol work, Hyprland/XDPH patches, PipeWire DMA-BUF fixes, NVIDIA SDK integration, `quinn` transport improvements, and Apple-client decode/render experiments are valid project work when they directly improve the reference environment.

## Non-Goals

- No Moonlight/GameStream compatibility.
- No X11-first architecture.
- No cloud pairing account or relay service in the core product.
- No generic remote-admin UX as the primary mode.
- No lowest-common-denominator multi-compositor abstraction before Hyprland is excellent.
- No leaking compositor-specific APIs across the host architecture. Hyprland-specific behavior belongs in a Hyprland adapter.
- No browser client as a design constraint.
- No WebRTC unless a measured requirement beats the QUIC design.
- No file transfer, clipboard, or controller feature that can compromise input/video latency.
- No indefinite waiting for upstream maturity when contribution, a narrow fork, or home-rolled code would materially advance the reference stack.
- No fallback path that changes security, capture scope, latency, color, sync, or privacy properties to accommodate older stacks.
- No permanent downstream fork unless upstream contribution is blocked for a documented technical or governance reason.
- No broad "Linux desktop" support claim before the Hyprland reference path is excellent.
- No older GPU, older Mac, older kernel, non-Hyprland compositor, or alternate distro support promise unless it emerges at no cost after the reference path is flawless.

These are rejected because they would force launch-record semantics, protocol constraints, cloud assumptions, or lowest-common-denominator capture/input behavior. UX lessons, diagnostics, controller mappings, stats overlays, and migration aids remain fair game.

## Reference Stack

madobe optimizes for the collaborators' actual machines:

- Host A: Linux 7.x+, NixOS, Hyprland, NVIDIA RTX 4090.
- Host B: Linux 7.x+, Kubuntu/Ubuntu, Hyprland, NVIDIA RTX 5060 Ti-class hardware.
- Client: M5 Pro MacBook-class macOS machines.
- Network: LAN or Tailscale direct.

Everything else is out of scope by default. Other users are welcome if they bring an equal-or-newer stack, but the project is not obligated to bend architecture, codec choices, capture paths, or latency targets for them.

## Target Users

Reference user:

- Runs a Linux 7.x+ NixOS or Kubuntu workstation with Hyprland.
- Uses NVIDIA RTX 4090/5060 Ti-or-newer-class hardware with AV1-capable NVENC.
- Uses an M5 Pro MacBook-class macOS client with AV1-capable VideoToolbox decode.
- Moves between coding, desktop work, media, games, and VM windows.
- Prefers local-first systems and will accept a narrow, modern setup if the result is excellent.

## Product Contract

When a client connects:

1. Client sends display/device capabilities: pixel size, scale, refresh, HDR/EDR support, codecs, audio devices, input devices, controllers, and supported clipboard/file modes.
2. Host evaluates local permission policy for the paired client and active session before enabling input, audio, clipboard, files, controller, or admin lanes.
3. Host creates or leases a named Hyprland headless output, for example `madobe-1`.
4. Host applies resolution, refresh, scale, bit depth, position, and workspace binding.
5. Host assigns a workspace or launched app group to that output.
6. Host captures only that output.
7. Host encodes the stream with low-latency NVENC settings.
8. Client decodes with VideoToolbox and renders with Metal.
9. Protocol lanes carry video, audio, input, controller state, clipboard, file transfer, telemetry, and admin operations.
10. Disconnect removes, parks, or recycles the output according to explicit session policy.

## Reference Architecture

```text
apps/
  hostd/             Rust user-session host daemon
  macos/             Swift/AppKit/SwiftUI shell with Metal view
  cli/               madobectl diagnostics and manual session control

crates/
  protocol/          Message schema, capability model, frame headers
  transport/         QUIC session, lane scheduling, auth/pairing
  compositor/        Capability model and host display/session traits
  compositor-hypr/   Hyprland IPC, output lifecycle, workspace/window reconciliation
  compositor-kwin/   KWin adapter backend isolated behind the same compositor contract
  capture/           PipeWire/ext-image-copy adapters and frame metadata
  encode-nv/         NVENC SDK wrapper, CUDA/Vulkan/EGL interop
  audio/             PipeWire audio capture and Opus encode
  input-linux/       uinput, evdev, libei/EIS experiments
  telemetry/         Latency stamps, counters, debug bundles

apple/
  ProtocolCore/      Optional Rust FFI or Swift-native protocol bindings
  MadobeMac/         Native macOS app

nix/
  module.nix         NixOS service/session integration
  flake.nix          Dev shell, formatter, package graph

docs/
  PRD.md
  research/
```

NixOS is the preferred reference deployment because it lets the project pin Hyprland, NVIDIA driver, PipeWire, xdg-desktop-portal-hyprland, GStreamer/FFmpeg, and SDK dependencies in a reproducible way. Kubuntu/Ubuntu is recognized only as the collaborator's second host shape, with the same modern capability expectations and no broader distro compatibility promise.

## Host-Side Technical Bets

### Compositor Adapter

The host daemon should depend on a small compositor adapter, not direct Hyprland calls. The adapter exposes product capabilities, not compositor trivia:

- create or lease a named remote output
- configure mode, scale, position, bit depth, and refresh where supported
- bind or activate a workspace/session on that output
- launch or move owned windows into that session
- enumerate outputs, workspaces, and windows for reconciliation
- remove, park, or recover owned remote outputs
- subscribe to output/window/workspace lifecycle events
- report capability gaps as typed errors, not silent degraded behavior

The adapter is not a portability promise. It is a blast-radius boundary. A compositor backend must satisfy the madobe session contract or it does not ship as a product backend.

### Hyprland Reference Backend

The headless display model is feasible now on Hyprland. Hyprland exposes fake output creation through `hyprctl output create headless <name>` and removal through `hyprctl output remove <name>`. It also exposes dynamic monitor rules such as `monitor = name, resolution, position, scale`, workspace binding, dispatchers, JSON state, and socket events. Hyprland itself recommends headless outputs for VNC/RDP/Sunshine-style use cases.

madobe should use a user-session daemon first, not a Hyprland plugin. The daemon should:

- Create named outputs: `madobe-1`, `madobe-2`.
- Immediately apply mode/scale/position far outside the physical layout, for example `50000x0`.
- Create persistent named workspaces bound to those outputs.
- Launch or move apps into the workspace, then reconcile with `hyprctl -j clients`.
- Subscribe to socket2 events and repair state after daemon restart.
- Keep a small warm pool of low-resolution parked outputs if creation latency or app migration is visible.

Direct Hyprland plugin work is a later optimization because Hyprland internals and plugins are tightly version-coupled. Generic wlroots `wlr-output-management` can configure existing heads but cannot create/destroy virtual heads, so it is not enough for the product contract.

### KWin Backend

KWin is a planned backend for the collaborator's KDE host, but it must not weaken the Hyprland reference path. Current KWin has virtual-output and screencast plumbing, PipeWire DMA-BUF/modifier negotiation, cursor metadata, damage metadata, and EIS input infrastructure. The main concern is control-surface stability: KWin virtual output creation is exposed through KDE-private unstable screencast protocols and tied to screencast lifecycle, unlike Hyprland's explicit daemon-friendly output lifecycle.

The KWin backend should be implemented behind the same compositor adapter contract. If KDE-private protocol behavior is insufficient, the response follows madobe's dependency doctrine: no-downside alternative, upstream contribution, narrow fork, home-rolled backend component, then diagnostic fallback. Required work:

- Own a named virtual output without weakening session semantics.
- Preserve DMA-BUF and explicit sync requirements; reject CPU readback as a product path.
- Control mode, scale, and refresh directly or through a clean KDE patch.
- Route windows/apps to the output without fragile user-workflow coupling.
- Map EIS or the chosen input path correctly to the remote output and survive reconnect.
- Keep KDE-private protocol handling contained in `compositor-kwin`.

### Capture Path

Primary production path:

```text
Hyprland headless output
  -> xdg-desktop-portal-hyprland / PipeWire ScreenCast
  -> DMA-BUF frame import
  -> GPU colorspace/format conversion
  -> NVENC
```

Portal/PipeWire is attractive because it is the modern permissioned screen capture path and exposes monitor/window/virtual source types, cursor modes, PipeWire streams, restore tokens, stream metadata, and `pipewire-serial` for stable stream targeting. PipeWire DMA-BUF support is the key requirement: the capture adapter must negotiate DRM formats/modifiers, require `SPA_DATA_DmaBuf` for product streaming, import buffers via graphics APIs, and avoid CPU readback.

A paired client must be able to reconnect to the same headless output without a fresh portal prompt on the reference host. If restore tokens or XDPH behavior cannot provide that, the capture strategy changes before app UX work proceeds.

Direct capture path:

- Treat `ext-image-copy-capture-v1` as the direct-capture optimization path when Hyprland/wlroots support is good enough.
- If XDPH/PipeWire cannot provide unattended, stable capture without CPU readback for named Hyprland outputs, contribute first; fork or home-roll a direct capture path before accepting SHM/CPU-copy as a default.

`wlr-screencopy` is useful historical context, not a product target. If current capture protocols cannot satisfy the product contract, madobe should push Hyprland/XDPH/wlroots forward or own the missing capture component.

Strict zero-copy is not the right promise. The practical promise is: no CPU readback after capture. The compositor will render/copy into a capture buffer; madobe must keep everything after that on GPU. SHM is a diagnostic tool only; it is not a release path.

Capture metadata contract:

- Log memory type, DRM device, DRM format, modifier, plane count, offsets, strides, color range, and sync mode for every stream.
- Prefer PipeWire DMA-BUF explicit sync through `SPA_META_SyncTimeline`; accept implicit sync only with telemetry and warning.
- Check `DRM_CAP_SYNCOBJ` and `DRM_CAP_SYNCOBJ_TIMELINE` before negotiating explicit sync.
- Every DMA-BUF frame needs an acquire/release story: explicit syncobj timeline preferred; implicit sync tolerated only while measured.
- Negotiate device identity to avoid cross-GPU import surprises.
- Use compositor damage and presentation timestamps where the capture path exposes them.

### Encode Path

Bring-up codec:

- H.264, 8-bit 4:2:0, no B-frames, low-latency CBR/VBR profile only as a bootstrap/debug codec if it helps isolate early pipeline bugs.

Reference codecs:

- AV1 is the reference codec for the known 4090/5060 Ti + M5 Pro environment. It should be part of the first real quality/latency target, not a distant stretch goal.
- HEVC Main/Main10 for quality, bandwidth, and HDR experiments.
- H.264 does not define the product and should not shape architecture.

Implementation ladder:

1. GStreamer spike for fast validation. Use PipeWire source and NVIDIA `nvcodec` elements where they can preserve GPU memory. Validate `zerolatency`, `CUDAMemory`/`GLMemory`, P010, and damage behavior.
2. Direct NVIDIA Video Codec SDK for the product path. The SDK exposes Linux NVENC control, CUDA/OpenGL device sessions, capability queries, presets, tuning info, input formats, timestamps, rate control, and newer SDK 13.x/13.1 features.
3. FFmpeg as a proof and regression tool, not the core path, because it is harder to guarantee no CPU hop from PipeWire capture to NVENC without custom integration.

RTX 4090 supports AV1 encode; RTX 50-series/Blackwell broadens encode/decode capabilities, including newer 4:2:2 paths. Optimize for the collaborators' 40/50-series + M5 Pro hardware immediately.

### Modern Kernel and Wayland Capabilities

madobe should deliberately target a pinned, current Linux graphics stack:

- July 2026 reference: Linux 7.1.x stable or newer. Longterm kernels are not a design target.
- The Nix matrix must pin kernel, NVIDIA driver branch, Hyprland, PipeWire, xdg-desktop-portal-hyprland, Mesa/Vulkan loader, CUDA, GStreamer/FFmpeg, and Video Codec SDK versions.
- `linux-dmabuf`: buffer sharing with explicit DRM format/modifier negotiation.
- DRM syncobj / explicit synchronization: avoid stale buffers, tearing, and implicit-sync driver ambiguity where supported.
- PipeWire 1.x DMA-BUF negotiation: require DMA-BUF for product streaming; SHM is diagnostic only.
- Wayland staging image capture protocols: track `ext-image-copy-capture-v1`.
- Hyprland color management and monitor `bitdepth, 10`: use only after capture correctness is proven.
- libei/EIS through XDG RemoteDesktop where it becomes viable for input injection.
- Linux 6.16-era DMA-BUF networking work, including device-memory TCP TX and io_uring zero-copy receive DMA-BUF support, is not an immediate QUIC dependency but is worth tracking for future GPU-resident transport experiments.
- Vulkan Video should be tracked as an encode/decode abstraction and validation path, but direct NVENC remains the host product path unless Vulkan Video becomes the better no-downside path on the reference stack.
- Vulkan should be the preferred DMA-BUF import/convert experiment because its external-memory and DRM-format-modifier extensions model Linux DMA-BUF directly; NVENC remains direct SDK through CUDA/OpenGL sessions until Vulkan can feed the encoder with equal control.

This is a Wayland-native system. XWayland apps can run inside the remote workspace, but X11 capture/input mechanisms are not first-class.

### Input

Initial Linux input injection should be pragmatic:

- Keyboard/pointer: `uinput` for controlled virtual devices, with explicit permission and diagnostics.
- Relative pointer: separate event type; do not fake relative motion by warping absolute position.
- Controller: virtual evdev/uinput gamepad devices, with a strict supported controller profile list.
- libei/EIS: serious spike through XDG RemoteDesktop v2 `ConnectToEIS`. Once EIS is connected, input must go through EIS rather than legacy portal `Notify*` methods. Do not block first product on it, but do treat it as the upstreamable Wayland-native direction.

Security model:

- Host daemon runs in the user session, not as a broad privileged root service.
- Any privileged helper is single-purpose, least-privilege, auditable, and has a small command surface.
- The helper must not own pairing, session policy, file access, or network listening.
- Input injection is scoped to one paired client and one active session.
- Permission revocation and disconnect must release all pressed keys/buttons and tear down virtual devices where practical.

### Audio

Host audio path:

- PipeWire capture of selected monitor/application/session audio.
- Opus baseline for low-latency audio.
- AAC optional later only if Apple-specific integration proves useful.
- Separate audio timestamps and drift correction, not video-clock guessing.

### Clipboard and Files

Clipboard:

- Text and URL first.
- Images second.
- Rich MIME after an explicit design.
- Size limits, direction controls, and visible permission state.

Files:

- Reliable QUIC streams.
- Per-file resumable chunks.
- Explicit transfer UI and policy.
- No virtual filesystem until the core stream is excellent.

### Color and HDR Contract

SDR correctness ships before HDR claims. HDR is not considered working until a captured PQ/HLG desktop round-trips to macOS EDR with measured luminance and correct metadata.

The protocol must be able to carry:

- Pixel format and bit depth.
- Full/video range.
- YCbCr/RGB matrix.
- Transfer function.
- Primaries.
- Mastering display metadata.
- MaxCLL and MaxFALL where known.
- SDR white level.
- EDR headroom.

Wayland color management should be tracked as protocol state, not reduced to Hyprland `bitdepth, 10`. Hyprland/output config, Wayland color-management state, encoder metadata, and macOS EDR state all need traceable handoff points.

## Transport and Protocol

Use one QUIC connection per viewer session with `quinn`.

Lanes:

| Lane | Transport primitive | Policy |
| --- | --- | --- |
| Control/admin | Reliable bidirectional stream | Auth, pairing, lifecycle, capabilities, display negotiation, close reasons |
| Video | QUIC DATAGRAM | Latest-frame semantics, frame/fragment ids, dependency ids, drop stale |
| Audio | QUIC DATAGRAM | Small jitter buffer, Opus packets, optional narrow redundancy |
| Input | Reliable stream, optionally datagram for high-rate mouse deltas after measurement | Ordered key/button safety; never blocked by files |
| Controllers | Datagram deltas plus reliable periodic full state | Hotplug, state repair, haptics later |
| Clipboard | Reliable stream | Transactional, size-limited, permissioned |
| Files | Reliable streams | Separate streams/chunks, low priority, cancelable |
| Telemetry | Datagram samples plus reliable summaries | Never competes with input/video |

QUIC gives TLS 1.3 identity, streams without cross-stream ordering, path migration, connection stats, and datagrams. QUIC does not give reliable-with-deadline partial reliability, so madobe implements deadline semantics at the application layer: old video and telemetry are discarded, stale object streams are reset, and decode state is recovered through keyframes or intra-refresh.

Congestion control:

- Start with CUBIC.
- Compare NewReno and experimental BBR in spikes.
- Add adaptive bitrate above transport, not inside the encoder alone.
- No generic FEC in v1. Add narrow audio redundancy or video keyframe/intra-refresh policies only after loss measurements.

Frame pacing policy:

- Use compositor presentation timestamps where available, capture timestamps, encoder submit/output timestamps, QUIC send time, client receive time, decode output time, and Metal present target/actual.
- Capture, encode, network, and decode queues must not exceed one frame unless explicitly in quality-recovery mode.
- Empty damage frames should be skipped when it saves encode/network work without harming cursor/input feedback.
- Pacing telemetry should distinguish dropped-before-encode, dropped-before-send, dropped-after-receive, decoder-late, and renderer-late frames.

Tailscale:

- Treat Tailscale as an underlay, not an auth system and not a latency guarantee.
- Surface direct, peer-relay, or DERP path state in telemetry.
- Degrade aggressively on DERP: lower resolution/FPS, disable bulk transfer, preserve input and audio.

Pairing:

- Local-first device identity.
- Host long-lived key/cert.
- Short-lived pairing code or QR.
- Client pins host identity.
- Tailscale identity may be an allowlist hint, never the sole trust boundary.

### Permission Model

Permissions are local-first, per paired client, per host user, and per active session.

- Default deny for input injection, clipboard sync, file transfer, controller injection, microphone/input audio if ever added, and admin operations.
- Video capture is limited to the madobe-owned headless output unless the user explicitly chooses another source.
- Clipboard and files have independent direction controls, size limits, and visible active state.
- Input permission can be revoked immediately and must trigger stuck-key/button release.
- Pairing grants identity, not capabilities. Capabilities are authorized by host policy at session start.
- Tailscale identity may narrow who can attempt pairing but must not bypass madobe authorization.
- Permission grants must be stored locally, inspectable, revocable, and included in debug bundles without leaking secrets.

## macOS Client Requirements

The first client should be native macOS, likely AppKit/Swift with SwiftUI where it helps. The render loop should not be built on `AVPlayer`.

Video:

- `VTDecompressionSession` direct decode.
- AV1 access units in `CMSampleBuffer` for the reference path.
- Set realtime decode behavior.
- Prefer hardware decode; require hardware only in strict diagnostics.
- Require hardware AV1 decode on the reference M5 Pro client path.
- Output `CVPixelBuffer` with Metal compatibility.
- Use `CVMetalTextureCache` and Metal shaders for YCbCr-to-RGB, scaling, cursor composition, and overlays.

HDR/EDR:

- SDR first.
- HDR spike uses HEVC Main10, `CAMetalLayer` extended dynamic range, 10-bit/float render targets, display capability checks, and explicit metadata validation.
- AV1 decode is part of the reference M5 Pro client path. Older Apple Silicon clients are not a design target.

Audio:

- CoreAudio low-latency output path.
- Small jitter buffer.
- Drift correction against stream timestamps.
- No blocking/allocation/locks in realtime callbacks.

Input:

- Focused-window AppKit `NSEvent` for MVP.
- Relative pointer from deltas while focused, with clear capture/release UX.
- Document shortcuts that macOS owns and cannot reliably forward.
- Global capture requires Input Monitoring/TCC and should be optional, not MVP-critical.
- GameController framework for gamepads.

The macOS client needs an explicit input behavior contract: forwarded shortcuts, locally handled shortcuts, remappable shortcuts, OS-reserved shortcuts, capture/release behavior, Secure Input failure mode, and game relative-pointer behavior.

Clipboard/files:

- `NSPasteboard` change-count sync with user-visible toggle and size limits.
- File transfer through explicit panels, drag/drop, and file promises.

iOS/iPadOS later:

- Same protocol and Metal/VideoToolbox core.
- Accept narrower input, foreground, filesystem, pasteboard, and backgrounding limits.

## Host MVP Gate

Do not begin broad app work until the host vertical slice passes these gates:

1. Create, resize, scale, and remove named Hyprland headless outputs repeatedly without compositor restart.
2. Bind persistent workspaces to those outputs and launch/move real apps into them.
3. Capture only the target output at 1440p60 first, then 4K60 on the same architecture.
4. Prove DMA-BUF capture with logged DRM format, modifier, device, sync behavior, and no CPU readback.
5. Prove every DMA-BUF frame has explicit or documented implicit acquire/release synchronization.
6. Encode AV1 on the RTX 4090/5060 Ti reference hosts and decode it on M5 Pro clients with capture-to-render p95 under the selected frame interval.
7. Use H.264 only if it accelerates debugging; it does not count as passing the product video gate.
8. Reconnect a paired client to the same headless output without a fresh portal prompt.
9. Send video over QUIC datagrams and prove stale frames are dropped rather than queued.
10. Decode/render on macOS with no more than one queued frame.
11. Inject keyboard, absolute pointer, relative pointer, and release stuck input safely.
12. Capture/play audio with bounded jitter and drift telemetry.
13. Produce one end-to-end telemetry trace with capture, encode, send, receive, decode, render, and input stamps.

If any MVP gate fails, product scope or implementation changes before feature work begins. Diagnostic escape hatches may unblock research, but they do not count as passing the gate.

## Telemetry Contract

Telemetry is not an overlay afterthought. Every frame and major event needs timestamps:

- Host capture start/end.
- GPU import/convert start/end.
- Encode submit/output.
- QUIC enqueue/send.
- Client receive/reassemble.
- Decode submit/output.
- Metal present target/actual.
- Input sample, host inject, visible response marker when available.
- Audio capture/playout drift.
- Color/HDR metadata handoff state.
- Sync mode and acquire/release wait/signal timing.

Session stats:

- RTT, jitter, congestion window, datagram drops, retransmits for reliable streams.
- Capture format/modifier/device and any diagnostic SHM use.
- Encoder queue depth, bitrate, QP or equivalent quality signal.
- Decoder queue depth and frame drop reason.
- Tailscale path state when discoverable.

## 2026 Capability Notes

- Kernel.org lists Linux 7.1.x stable and 7.2-rc mainline as of July 2026; madobe targets Linux 7.x+.
- NVIDIA Wayland explicit sync requires syncobj fixes present in modern kernels; Linux 7.x+ satisfies this reference assumption.
- PipeWire documents DMA-BUF explicit sync through `SPA_META_SyncTimeline`; madobe should probe it, not assume it.
- XDG ScreenCast v6 clients should prefer `pipewire-serial` over PipeWire node ids for stream targeting.
- NVIDIA Video Codec SDK 13.1 is current, with newer AV1/UHQ/CUarray and pipeline optimization features worth testing but not blindly enabling.
- NvFBC is not a Wayland capture path.
- Vulkan Video AV1 encode/decode is worth tracking and validating, but direct NVENC remains the host product path.
- M5 Pro clients are in the reference path with hardware AV1 decode. Older Apple Silicon is not a release target.

## OSS Governance

- License: choose the license explicitly before accepting outside code.
- Contributions: use DCO or CLA, documented coding standards, and public architecture decisions for major subsystem choices.
- Security: publish supported versions, disclosure contact, embargo expectations, and vulnerability handling for auth, pairing, permissions, capture isolation, file transfer, and privileged helpers.
- Upstream policy: document when madobe contributes upstream, carries a patch, forks, or replaces a dependency.
- Dependency policy: prefer maintained libraries with compatible licenses, active security posture, and packageability in Nix.
- Compatibility policy: reject platform expansion that weakens the reference Hyprland/NVIDIA/macOS loop before it is reliable.

## Risk Register

| Risk | Impact | Default response |
| --- | --- | --- |
| PipeWire DMA-BUF modifier mismatch | CPU copy or broken capture | Fix upstream, fork, or home-roll; SHM is diagnostic only |
| Portal prompts block unattended UX | Bad "just works" experience | Restore tokens first, then contribute/fork/home-roll direct capture before accepting prompt-heavy UX |
| Hyprland output lifecycle changes | Host breakage | Pin versions in Nix, robust diagnostics, no plugin in MVP |
| Explicit sync gaps | Stalls or stale frames | Detect sync support, log path, prefer modern driver stack |
| NVIDIA driver regressions | Broken encode/capture | Pin known-good Linux 7.x/NVIDIA/Hyprland matrix and move forward deliberately |
| HDR metadata loss | Incorrect HDR | Separate HDR spike, do not market until verified |
| macOS input limitations | User confusion | Focused capture first, clear release UX, document OS-owned shortcuts |
| QUIC datagram loss under DERP | Latency/quality drop | Adaptive bitrate, keyframe repair, degrade bulk features |
| File transfer competes with stream | Worse interactivity | Do not ship file transfer until lane priority proves harmless |
| Multi-client isolation leaks | Privacy/security issue | Single-client MVP, explicit session ownership before multi-client |
| Bespoke implementation outruns upstream alternatives | Maintenance burden and slower delivery | Require build-vs-use decision records for transport, capture, encode, input, and packaging |
| Fork becomes permanent | Security and compatibility drag | Fork only with exit criteria, upstream issue/PR link, and owner |
| Diagnostic path becomes product path | Users get inconsistent latency/security | Keep diagnostics out of release claims unless explicitly promoted by decision record |
| Pairing confused with authorization | Paired client gets excessive power | Pairing grants identity only; permissions are separate and revocable |
| Privileged helper grows scope | Local privilege escalation risk | Single-purpose helper, narrow IPC, fuzzable parser, no network or policy authority |
| Clipboard leaks secrets | Passwords/tokens cross devices unexpectedly | Default off or prompt-first, direction controls, size/type limits, visible active state |
| File transfer path traversal or overwrite | Host/client data loss or compromise | Canonicalize paths, explicit destination, no implicit overwrite, quarantine/download semantics |
| Capture source escapes headless output | Privacy breach | Assert selected source identity, show capture target, fail closed on ambiguity |
| Debug bundles leak secrets | Support artifact becomes sensitive | Redaction policy, local preview, explicit user export |
| OSS governance ambiguity | Contributor conflict and licensing risk | Pick license, contribution terms, security policy, and maintainer authority early |

## Pre-DAG Research Areas

These are not qd nodes, phases, or backlog items. They are areas to settle before actionable qd work is created.

- Compositor contract: capability trait, typed errors, event model, and host session contract without leaking Hyprland details.
- Hyprland output lifecycle: `madobe-*` outputs, workspace binding, resize, warm pool, and restart repair.
- KWin output lifecycle: same output/session contract, including virtual output ownership, DMA-BUF capture constraints, window routing, input mapping, and reconnect behavior.
- Capture: PipeWire/XDPH capture of one headless output with DMA-BUF metadata and no CPU readback.
- Encode: direct SDK AV1 reference path; H.264 only as a debugging comparator; HEVC Main10 probe for HDR.
- Vulkan interop: import DMA-BUF with Vulkan, convert color/format, bridge to CUDA/NVENC, and measure any unavoidable copy.
- QUIC media: datagram frame transport, loss tests, reconnection, migration, and bulk contention.
- macOS decode/render: VideoToolbox + Metal AV1 decode/render on M5 Pro; HEVC Main10 probe for HDR.
- Linux input: uinput keyboard/mouse, relative pointer, gamepad skeleton, EIS path, and stuck-key recovery.
- Audio: PipeWire audio capture to Opus packets and macOS playout.
- `trace`: unified timestamp schema and debug bundle.

## First Release Shape

The first credible release is not feature-complete. It is a low-latency single-client session:

- NixOS reference host.
- Kubuntu/Ubuntu collaborator host on the same modern capability matrix.
- One Hyprland headless output.
- One macOS client.
- 1440p60 SDR AV1 on known-good RTX 4090/5060 Ti hosts and M5 Pro clients.
- 4K60 AV1 if the same architecture meets latency and stability gates.
- Keyboard, absolute pointer, relative pointer.
- Stereo audio.
- Basic pairing.
- Stats overlay.
- Debug bundle.

Everything else is conditional:

- Dynamic resize only after capture/encode/client renegotiation does not hitch badly.
- Clipboard text after input/video/audio are stable.
- File transfer after lane priority proves safe.
- Controllers after keyboard/mouse are safe.
- HEVC Main10 before HDR.
- Multi-monitor and multi-client only after single-session ownership is boringly reliable.

Daily usability gates for the first release:

- Reconnect after client sleep, network flap, and app relaunch.
- Recover after host daemon restart by reconciling outputs, workspaces, windows, and active sessions.
- Clear stale outputs/workspaces without harming user-owned windows.
- Detect incompatible host/client protocol versions before session start.
- Produce actionable diagnostics for reference-stack mismatches, missing permissions, CPU-readback capture, portal failures, and Tailscale relay paths.
- State clearly that v1 is for LAN or Tailscale direct paths and does not claim general remote access across arbitrary locked-down networks.

## Source Anchors

- Hyprland `hyprctl` output creation, monitor config, IPC, and NVIDIA notes: <https://wiki.hypr.land/0.49.0/Configuring/Using-hyprctl/>, <https://wiki.hypr.land/0.49.0/Configuring/Monitors/>, <https://wiki.hypr.land/IPC/>, <https://wiki.hypr.land/Nvidia/>
- XDG ScreenCast and RemoteDesktop portals: <https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html>, <https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html>
- PipeWire DMA-BUF buffer data type and negotiation metadata: <https://docs.pipewire.org/group__spa__buffer.html>
- Kernel and NVIDIA Wayland constraints: <https://www.kernel.org/>, <https://www.kernel.org/category/releases.html>, <https://download.nvidia.com/XFree86/Linux-x86_64/575.64.05/README/wayland-issues.html>
- Wayland capture/sync/color/pacing protocols: <https://wayland.app/protocols/wlr-screencopy-unstable-v1>, <https://wayland.app/protocols/ext-image-copy-capture-v1>, <https://wayland.app/protocols/linux-drm-syncobj-v1>, <https://wayland.app/protocols/color-management-v1>, <https://wayland.app/protocols/presentation-time>
- NVIDIA Video Codec SDK and support matrix: <https://developer.nvidia.com/video-codec-sdk>, <https://docs.nvidia.com/video-technologies/video-codec-sdk/13.0/nvenc-video-encoder-api-prog-guide/index.html>, <https://developer.nvidia.com/video-encode-decode-support-matrix>
- QUIC and `quinn`: <https://docs.rs/quinn/latest/quinn/>, <https://www.rfc-editor.org/rfc/rfc9000.html>, <https://www.rfc-editor.org/rfc/rfc9221.html>, <https://www.rfc-editor.org/rfc/rfc9002.html>
- Tailscale connection behavior: <https://tailscale.com/docs/reference/connection-types>, <https://tailscale.com/docs/reference/faq/firewall-ports>
- Apple client anchors: <https://developer.apple.com/documentation/videotoolbox>, <https://developer.apple.com/documentation/metal>, <https://developer.apple.com/documentation/gamecontroller>, <https://developer.apple.com/documentation/appkit/nspasteboard>, <https://www.apple.com/newsroom/2023/10/apple-unveils-m3-m3-pro-and-m3-max-the-most-advanced-chips-for-a-personal-computer/>, <https://www.apple.com/newsroom/2024/05/apple-introduces-m4-chip/>
- Existing projects to study, not inherit: <https://github.com/LizardByte/Sunshine>, <https://github.com/moonlight-stream/moonlight-ios>, <https://github.com/games-on-whales/wolf>
