# Changelog

All notable changes to Cerul Robotics are recorded here. Release automation
publishes the section that matches the tagged version as the GitHub release
notes, so keep an entry under **Unreleased** for every user-visible change and
rename that heading when a version is cut.

## 0.1.0 - 2026-10-07

First release as a separate product. These workflows previously shipped inside
the `cerul` CLI.

### Added
- `cerul-robotics annotate`: subtask, event, interaction and state labels for
  videos and LeRobot datasets, plus optional local human-hand keypoints with
  `--hands`. Hand models are embedded; no model key is needed for
  `--hands --semantic none`.
- `cerul-robotics render`: annotated review videos with no model calls.
- `cerul-robotics index`, `analyze`, `search` and `status` over LeRobot
  datasets and ordinary videos, using the shared Cerul engine.
- LeRobot v3.1 subtask writeback with `--write-lerobot`, preferably to a new
  dataset with `--out`.
- One-line installer for macOS Apple Silicon and Linux x86_64. Compatible media
  tools are downloaded and checksum-verified on first use; `--no-auto-deps`
  disables this.
- Agent skill, installable with `npx skills add cerul-ai/cerul-robotics` or
  printed by `cerul-robotics skill`.

### Notes
- The default workspace is `~/.cerul-robotics`. Labels made with
  `cerul annotate` remain in the Cerul workspace; pass `--workspace ~/.cerul` to
  read them.
- Generated labels are model output, not human-verified ground truth.
