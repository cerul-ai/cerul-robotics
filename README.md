# Cerul Robotics

Search, annotate and review robot episodes using your own model endpoints.

Cerul Robotics owns LeRobot dataset adapters and writeback, semantic task labels,
local human-hand tracking, and annotated review videos. It depends on the
[Cerul](https://github.com/cerul-ai/cerul) library for media processing, providers,
OCR, retrieval and authoritative sidecars. It does not duplicate the core engine.

## Install

Supports **macOS Apple Silicon** and **Linux x86_64**.

```sh
curl -fsSL https://github.com/cerul-ai/cerul-robotics/releases/latest/download/cerul-robotics-installer.sh | sh
cerul-robotics --version
```

The release is one self-contained binary with the hand models embedded. No Rust,
Python, GPU runtime or system FFmpeg is required: compatible media tools are
downloaded and checksum-verified on first use. Set `CERUL_FFMPEG` and
`CERUL_FFPROBE`, or pass `--no-auto-deps`, to use your own.

To let a coding agent drive it, add the skill:

```sh
npx skills add cerul-ai/cerul-robotics
```

Building from source instead needs a stable Rust toolchain and protobuf:
`cargo install --git https://github.com/cerul-ai/cerul-robotics --locked`.

## Start with one episode

Upgrading from `cerul annotate`? Your earlier labels live in the Cerul workspace.
Add `--workspace ~/.cerul` to `status`, `search` and `render` to read them.

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
