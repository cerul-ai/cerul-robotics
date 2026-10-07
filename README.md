# Cerul Robotics

Search, annotate and review robot episodes using your own model endpoints.

Cerul Robotics owns LeRobot dataset adapters and writeback, semantic task labels,
local human-hand tracking, and annotated review videos. It depends on the
[Cerul](https://github.com/cerul-ai/cerul) library for media processing, providers,
OCR, retrieval and authoritative sidecars. It does not duplicate the core engine.

## Install

Requires a stable Rust toolchain, protobuf for the build, and compatible FFmpeg
and ffprobe executables at runtime. No Python or GPU runtime is required to use
the CLI. Python is needed only for official LeRobot loader acceptance tests.

```sh
cargo install --git https://github.com/cerul-ai/cerul-robotics --locked
cerul-robotics --help
```

The initial extraction is delivered through a pull request. Installation from
`main` becomes available after that PR merges; no binary release is claimed yet.

## Start with one episode

Set your model provider key securely in the environment, or reuse an existing
endpoint-scoped key saved by `cerul auth set`. Configuration follows Cerul:
`~/.cerul-robotics/config.toml`, project `cerul-robotics.toml`, environment, then `--set` overrides.
Provider requests may send sampled media to the configured endpoint.

```sh
cerul-robotics annotate ./dataset --only 0 --semantic --dry-run --json
cerul-robotics annotate ./dataset --only 0 --semantic --json
cerul-robotics render ./dataset/.cerul/episodes/0/annotations.json --out ./review.mp4
cerul-robotics index ./dataset --only 0 --json
cerul-robotics search --filter semantic.event.verb=grasp --json
```

Ordinary video files are supported too. The workspace defaults to `~/.cerul-robotics` and
is separate from Cerul. Use `CERUL_ROBOTICS_WORKSPACE` or `--workspace DIR` to
select another registry, including an existing Cerul workspace for migration. Use `status`
to locate published results. Human output is structured JSON; `--json` emits one
compact final object on stdout and NDJSON events on stderr. Exit 6 means partial
work, 5 cancellation and 3 unsupported capability.

`--hands --semantic none` runs local human-hand inference without a model key.
Human hands are not robot grippers. Depth and calibrated 3D poses are unsupported.
Generated labels are not human-verified ground truth or evidence of policy quality.

## Data safety

Sidecars remain authoritative and indexes are rebuildable caches. Integer
microseconds, content-aware invalidation, atomic publication and recoverable
writeback are preserved. Rerun interrupted commands to reuse completed work.

Dataset mutation is opt-in through `--write-lerobot`; prefer `--out NEW_DIRECTORY`.
Read [compatibility and writeback](docs/lerobot.md) before using it. Review videos
never overwrite an existing output. Existing Cerul annotation files remain readable.

## Guides

- [Annotation](docs/annotation.md)
- [LeRobot subtasks](docs/lerobot-subtasks.md)
- [Validation and release acceptance](docs/development/validation.md)
- [Agent skill](skills/cerul-robotics/SKILL.md), also printed by `cerul-robotics skill`
- [Source provenance](PROVENANCE.md)
- [Licenses and model provenance](THIRD_PARTY_NOTICES.md)

## Development

```sh
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run --locked --example generate_schemas -- --check
```

The shared engine dependency is pinned to an exact Git revision. Update that pin
explicitly and rerun cross-repository validation. This is an independent product
release; it does not require copying or publishing the core CLI.
