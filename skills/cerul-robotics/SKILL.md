---
name: cerul-robotics
description: Annotate, search and review robot episodes and LeRobot datasets using the Cerul Robotics CLI.
---

# Cerul Robotics

## Before anything else

Run `cerul-robotics --version`. If the command is missing, install the published
release; it is a single binary and needs no Rust, Python or system FFmpeg:

```sh
curl -fsSL https://github.com/cerul-ai/cerul-robotics/releases/latest/download/cerul-robotics-installer.sh | sh
```

Do not build from source or install a Rust toolchain to work around a failed
download; report the error instead. Media tools are fetched and verified on first
use unless `--no-auto-deps` is set.

## Working with episodes

Run `cerul-robotics --help` and command help before processing. Start with one episode and `--dry-run`. Semantic annotation uses the configured vision endpoint; do not call generated labels human-verified ground truth. Never infer trained policy quality from annotation output.

Use `cerul-robotics annotate DATASET --only 0 --semantic --json` for subtasks, events, interactions and states. Use `--hands --semantic none` for local human-hand keypoints only. Robot gripper detection, depth and calibrated 3D poses are not implemented.

Published `annotations.json` and `summary.md` preserve source provenance. Use `cerul-robotics render ANNOTATIONS_JSON --out NEW_MP4` to review them offline. Never overwrite original media. `--write-lerobot` explicitly opts into dataset mutation; prefer `--out NEW_DATASET` and read the LeRobot compatibility guide first.

Use `index` for dataset search, `search` for natural-language queries or typed filters, `analyze` for general scene summaries, and `status` to inspect existing results. The workspace defaults to `~/.cerul-robotics`; use `--workspace` to explicitly reuse an existing Cerul registry. Endpoint fields and environment overrides use the shared Cerul format. The CLI can read the endpoint-scoped key saved by `cerul auth set` without modifying it. Model requests may send media to the configured endpoint.

With `--json`, stdout contains one final JSON object; stderr contains NDJSON events. Exit 6 means partial work, 5 cancellation, 3 unsupported capability. Rerun interrupted commands without `--recompute` to reuse completed work. Ranking scores are not confidence or accuracy.
