# M1 Audit Notes

Audit timestamp: 2026-07-03T19:12:02Z

Scope:

- M1 Rust/compositor boundary after merged implementation nodes.
- Public M1 evidence and qd reports.
- M1 DAG shape before milestone closeout.
- Linux hardware evidence for remote output lifecycle ownership.

Audit inputs:

- `qd status --json`: 25 nodes, 16 done, one claimed M1 audit node, one blocked M2 capture node, zero open P0/P1 findings.
- `qd ready --json`: no ready nodes while the audit claim is active.
- Prior M1 completion and audit reports under `reports/qd/m1-*`.
- Hardware lifecycle evidence under `evidence/m1-linux-hardware-validation`.
- Live non-mutating CLI check after boundary cleanup.

Resolved findings:

- `hostd` directly depended on `madobe-hyprland` and exposed `run_hyprland_display_action`. The concrete Hyprland adapter construction now lives in the `madobectl` binary composition root. `hostd` remains bound to the shared `CompositorAdapter` contract.
- Public M1 evidence had raw workspace window addresses/titles, local hostnames, and local user paths in older command and workspace artifacts. The audit branch redacts those values and normalizes prior M1 evidence.

Dispositioned findings:

- M1 lifecycle evidence proves create, adopt/configure, park, remove, and cleanup on Linux hardware.
- Workspace/session binding is implemented in the Hyprland adapter and covered by unit-level behavior, but it has not been hardware-validated with a live workspace moved to a madobe-owned remote output. This should become a P1 validation node before M1 is considered closed.
- Hyprland socket2 event fixture polish remains useful but is not required for M1 closeout because output ownership does not currently rely on event-driven reconciliation.

Boundary decision:

`apps/cli/src/main.rs` is treated as the Linux CLI composition root. It may construct the concrete Hyprland adapter for the executable path. Libraries and shared crates must stay backend-neutral unless their crate is explicitly the Hyprland adapter crate.
