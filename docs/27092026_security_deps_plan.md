# Security dependency update plan (27 Sep 2026)

## Goal
Clear the Dependabot alert opened when alerts were enabled on 27 Sep 2026, without changing app code.

## Status
In review on branch `chore/security-deps-27092026`.

## Milestones
- [x] Alert 1, `bytes` 1.11.0 (GHSA-434x-w66g-qw3r, integer overflow in `BytesMut::reserve`). Pulled in only by `tokio`. `cargo update -p bytes` moves it to 1.12.1; the lockfile diff is identical to Dependabot PR #1.
- [x] `cargo build --workspace --locked` and `cargo test --workspace --locked` pass (8 tests, macOS arm64, rustc 1.95.0). The tests do not touch Bluetooth or HID hardware.
- [ ] CI green on the PR and on `main` after merge; open alerts re-queried.

## Decisions
- Dependabot PR #1 was opened before CI existed on `main`, so it has no CI result. This PR carries the same lockfile change plus docs; #1 will be closed as superseded once this merges.

## Deviations
None.
