# Repository guidelines

Communicate with the owner in Chinese. Public documentation, code, comments,
identifiers and commit messages are English.

This repository owns robot episode annotation, human-hand inference, LeRobot
adapters/writeback and review-video rendering. Reuse the pinned Cerul library
for media processing, providers, OCR, annotations, sidecars and search. Do not
copy its implementation or introduce a circular dependency. Keep one root Rust
crate with a library and a thin CLI; no product UI, billing or hosted services.

Preserve integer-microsecond time, content-aware invalidation, atomic
publication, interruption recovery and zero-model-call index rebuilds. Library
code never parses arguments, prints progress or terminates the process.

Run cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings and
cargo test --locked. Generate schemas from Rust types. Release acceptance also
requires macOS arm64/Linux x86_64, real hand inference, actual model endpoint
smokes, interruption recovery and official LeRobot loader round-trips. See
docs/development/validation.md. Mock-only checks do not replace release gates.

Never commit secrets, user media, indexes or production exports. Preserve
LICENSE and model/fixture provenance. Retain original Cerul history, tags and
ffmpeg-vendor release assets. Never delete local worktrees or user artifacts.
Use main as the only long-lived branch and codex/ for agent branches. Public
merges go through ready-for-review PRs. Report only P0/P1 code-review findings.
