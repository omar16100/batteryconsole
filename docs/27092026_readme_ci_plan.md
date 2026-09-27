# README accuracy, docs scaffold and CI (27 Sep 2026)

## Goal

Make the README match the code and the published Homebrew formula, add the standard docs scaffold, and add a first CI workflow so pull requests get a real build and test signal.

## Status

Done in branch `docs/readme-ci-27092026`.

## Milestones

- [x] README: clone URL `yourusername` replaced with `omar16100`.
- [x] README: Homebrew is live (`brew install omar16100/tap/batteryconsole`, formula builds tag `v0.1.0`, tarball sha256 verified against the formula on 27 Sep 2026). Section no longer says "coming soon".
- [x] README: removed claims the repo does not back ("Fast and lightweight", "same method macOS System Settings uses", "macOS 12.0 or later"). Device list split into what was verified (MX Keys, MX Master 3 per `todo.md`) and what `src/devices.rs` names. Noted that BLE mode lists any peripheral with a Battery Service.
- [x] `docs/index.md`, `docs/c4model.md`, this plan.
- [x] `.github/workflows/ci.yml`: `macos-latest`, `cargo build --workspace --locked`, `cargo test --workspace --locked`.

## Decisions

- CI runs tests, not only a build: the 5 unit tests and 3 CLI tests need no battery, Bluetooth or HID hardware (checked locally: 8 passed).
- `--locked` is used because `Cargo.lock` is committed.
- Topic `tui` not added: the tool prints plain lines, it has no terminal UI.

## Deviations

- Repository topics set to `macos`, `battery`, `rust`, `cli`, `logitech`, `bluetooth`. A `tui` topic was proposed but not added, because there is no terminal UI.
