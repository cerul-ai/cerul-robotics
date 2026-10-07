# Robotics validation

Run `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
`cargo test --locked` and `cargo run --locked --example generate_schemas -- --check`.
CI runs on macOS arm64 and Linux x86_64 and exercises real embedded hand inference,
semantic recovery, transactions and offline rendering. These tests establish
behavior, not labeling accuracy or paid-production readiness.

Before publishing a version, record the exact core and Robotics revisions and
verify model endpoints against real requests, annotated video review, cancellation
and resume, dataset identity/content invalidation and zero-model-call rebuilds.
The official loader round-trip below is mandatory and is not replaced by Rust
fixture/schema-only checks. Core remains responsible for its OCR acceptance.

## Official LeRobot loader

The inherited acceptance harness targets Hugging Face LeRobot commit
[`2774d9bddcbbda50e697e162e89e7eaada8d7105`](https://github.com/huggingface/lerobot/tree/2774d9bddcbbda50e697e162e89e7eaada8d7105).
Its recorder emits v3.0 while exposing the language-column types used by the
synthetic v3.1 compatibility fixture. These tests do not establish the existence
of an official v3.1 recorder or an upgrade tool. See the
[user-facing compatibility limits](../lerobot.md).

Before a release, install the pinned loader in an isolated Linux environment:

```sh
python3.12 -m venv /tmp/cerul-loader-env
/tmp/cerul-loader-env/bin/python -m pip install 'torch==2.11.0' 'torchvision==0.26.0' --index-url https://download.pytorch.org/whl/cpu
/tmp/cerul-loader-env/bin/python -m pip install 'lerobot[dataset,av-dep] @ git+https://github.com/huggingface/lerobot.git@2774d9bddcbbda50e697e162e89e7eaada8d7105'
```

Use fresh source and output paths for both cases:

```sh
export HF_HUB_OFFLINE=1 HF_DATASETS_OFFLINE=1
/tmp/cerul-loader-env/bin/python tests/lerobot_roundtrip.py create /tmp/cerul-source
cargo run --locked --example verify_lerobot_writeback -- /tmp/cerul-source /tmp/cerul-output /tmp/cerul-loader-env/bin/python
/tmp/cerul-loader-env/bin/python tests/lerobot_roundtrip.py create /tmp/cerul-source-no-language --without-language
cargo run --locked --example verify_lerobot_writeback -- /tmp/cerul-source-no-language /tmp/cerul-output-no-language /tmp/cerul-loader-env/bin/python
```

The five-episode, 40-frame fixture covers existing and absent language columns.
The harness compares original fields, decoded images, actions, state, untouched
episodes, and active subtask timestamps. A rejected validator must leave the
output unpublished. Python is a test dependency, not a Cerul runtime dependency.

