use serde_json::Value;
use std::{
    path::Path,
    process::{Command, Output},
};

fn cli(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cerul-robotics"))
        .current_dir(directory)
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap())
        .env("HOME", directory)
        .env("CERUL_VISION_ENABLED", "false")
        .args(args)
        .output()
        .unwrap()
}
fn final_json(output: &Output) -> Value {
    let text = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(text.lines().count(), 1, "{text}");
    serde_json::from_str(text).unwrap()
}

fn video(directory: &Path) {
    let output = Command::new("ffmpeg")
        .args([
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=64x64:rate=2:duration=2",
            "-c:v",
            "libx264",
        ])
        .arg(directory.join("sample.mp4"))
        .output()
        .unwrap();
    assert!(output.status.success());
}
#[test]
fn annotation_help_teaches_the_workflow_without_loading_configuration() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("cerul.toml"), "invalid = [").unwrap();
    for args in [vec!["annotate"], vec!["annotate", "--help"]] {
        let output = cli(dir.path(), &args);
        let text = if args.len() == 1 {
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            String::from_utf8_lossy(&output.stderr)
        } else {
            assert!(output.status.success());
            String::from_utf8_lossy(&output.stdout)
        };
        assert!(text.contains("subtask,event,interaction,state"), "{text}");
        assert!(text.find("Examples:").unwrap() < text.find("Options:").unwrap());
        assert!(text.contains("--semantic --only 0"), "{text}");
        assert!(text.contains("no index step needed"), "{text}");
        assert!(text.contains("semantic.<type>.jsonl"), "{text}");
    }
    let output = cli(dir.path(), &["--json", "annotate"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(final_json(&output)["error"]["code"], "invalid_arguments");
    assert!(output.stderr.is_empty());
    let output = cli(dir.path(), &["annotate", "--semantci"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unexpected argument"));
    assert!(!dir.path().join(".cerul").exists());
    assert!(!dir.path().join(".cerul-robotics").exists());
}

#[test]
fn annotate_default_plan_and_m2_rejection_are_explicit() {
    let dir = tempfile::tempdir().unwrap();
    video(dir.path());
    assert_eq!(
        cli(
            dir.path(),
            &[
                "--json",
                "annotate",
                "sample.mp4",
                "--semantic",
                "none",
                "--dry-run"
            ]
        )
        .status
        .code(),
        Some(2)
    );
    for (extra, count) in [
        (vec!["--hands"], 5),
        (vec!["--embodied", "--hands"], 5),
        (vec!["--embodied", "--hands", "--semantic", "none"], 1),
    ] {
        let mut args = vec!["--json", "annotate", "sample.mp4", "--dry-run"];
        args.extend(extra);
        let output = cli(dir.path(), &args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        let value = final_json(&output);
        assert_eq!(value["modules"].as_array().unwrap().len(), count);
        assert_eq!(value["modules"][0]["annotation"], "grounding.hand");
    }
    let output = cli(
        dir.path(),
        &["--json", "annotate", "sample.mp4", "--dry-run"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let result = final_json(&output);
    let mut names: Vec<_> = result["modules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["annotation"].as_str().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "semantic.event",
            "semantic.interaction",
            "semantic.state",
            "semantic.subtask"
        ]
    );
    assert!(!dir.path().join(".cerul").exists());
    assert!(!dir.path().join(".cerul-robotics").exists());
    assert!(!dir.path().join("sample.mp4.cerul").exists());
    let selected = cli(
        dir.path(),
        &[
            "--json",
            "annotate",
            "sample.mp4",
            "--embodied",
            "--dry-run",
        ],
    );
    assert!(selected.status.success());
    let result = final_json(&selected);
    let mut names: Vec<_> = result["modules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["annotation"].as_str().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names,
        vec![
            "semantic.event",
            "semantic.interaction",
            "semantic.state",
            "semantic.subtask"
        ]
    );
    assert!(!dir.path().join(".cerul").exists());
    assert!(!dir.path().join(".cerul-robotics").exists());
    assert!(!dir.path().join("sample.mp4.cerul").exists());
    let output = cli(dir.path(), &["--json", "annotate", "sample.mp4", "--world"]);
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(final_json(&output)["error"]["code"], "missing_capability");
}

#[test]
fn invalid_ontology_is_a_configuration_error_without_workspace_writes() {
    use std::fs;
    let dir = tempfile::tempdir().unwrap();
    for (name, contents) in [
        ("empty.txt", ""),
        ("broken.json", "[\"reach\", "),
        ("object.json", "{}"),
    ] {
        fs::write(dir.path().join(name), contents).unwrap();
    }
    for name in ["empty.txt", "broken.json", "object.json", "missing.txt"] {
        let output = Command::new(env!("CARGO_BIN_EXE_cerul-robotics"))
            .current_dir(dir.path())
            .env_clear()
            .env("HOME", dir.path())
            .env("PATH", "")
            .args([
                "--json",
                "annotate",
                "missing.mp4",
                "--semantic",
                "--ontology",
                name,
            ])
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(final_json(&output).get("error").is_some());
        assert!(!dir.path().join(".cerul").exists());
        assert!(!dir.path().join(".cerul-robotics").exists());
    }
}

#[test]
fn a_partial_annotation_carries_the_command_that_continues_it() {
    let dir = tempfile::tempdir().unwrap();
    video(dir.path());
    let media = dir.path().join("sample.mp4");
    // Port 9 discards connections, so the vision endpoint fails after the local
    // work succeeds: the run is partial, which is what carries a retry.
    let output = Command::new(env!("CARGO_BIN_EXE_cerul-robotics"))
        .current_dir(dir.path())
        .env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap())
        .env("HOME", dir.path())
        .env("GEMINI_API_KEY", "test-key")
        .args([
            "--json",
            "annotate",
            media.to_str().unwrap(),
            "--semantic",
            "subtask",
            "--jobs",
            "1",
            "--set",
            "vision.base_url=\"http://127.0.0.1:9\"",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(6));
    let value = final_json(&output);
    assert_eq!(value["partial"], true);
    // The result describes its own recovery, in the schema, not beside it.
    let argv: Vec<String> = value["retry"]["argv"]
        .as_array()
        .expect("a partial run offers a retry")
        .iter()
        .map(|argument| argument.as_str().unwrap().to_owned())
        .collect();
    // The program is repeated as it was invoked: a path was used because the
    // binary is not on PATH, and shortening it would break the copied command.
    assert_eq!(argv[0], env!("CARGO_BIN_EXE_cerul-robotics"));
    assert_eq!(value["retry"]["reason"], "incomplete");
    // Every choice survives, or running it again would not be the same run.
    for argument in [
        media.to_str().unwrap(),
        "--semantic",
        "subtask",
        "vision.base_url=\"http://127.0.0.1:9\"",
    ] {
        assert!(argv.iter().any(|value| value == argument), "{argv:?}");
    }
    assert!(!argv.iter().any(|value| value == "--recompute"), "{argv:?}");
    // A source that is only a file name cannot say which input it came from,
    // and two directories can hold the same name.
    let source = value["modules"][0]["source"].as_str().unwrap();
    assert!(source.ends_with("/sample.mp4"), "{source}");
    assert!(std::path::Path::new(source).is_absolute(), "{source}");
}

#[test]
fn product_configuration_is_isolated_and_json_errors_are_structured() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".cerul")).unwrap();
    std::fs::write(dir.path().join(".cerul/config.toml"), "broken = [").unwrap();
    std::fs::write(dir.path().join("cerul.toml"), "broken = [").unwrap();
    let output = cli(dir.path(), &["--json", "status"]);
    assert!(output.status.success(), "{output:?}");
    std::fs::write(dir.path().join("cerul-robotics.toml"), "broken = [").unwrap();
    let output = cli(dir.path(), &["--json", "status"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        final_json(&output)["error"]["code"],
        "invalid_configuration"
    );
    assert!(!dir.path().join(".cerul-robotics").exists());
}

#[test]
fn media_commands_check_tools_first_and_never_download_when_disabled() {
    let dir = tempfile::tempdir().unwrap();
    video(dir.path());
    let output = Command::new(env!("CARGO_BIN_EXE_cerul-robotics"))
        .current_dir(dir.path())
        .env_clear()
        .env("PATH", dir.path())
        .env("HOME", dir.path())
        .args([
            "--json",
            "--no-auto-deps",
            "annotate",
            "sample.mp4",
            "--hands",
            "--semantic",
            "none",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let error = final_json(&output);
    assert_eq!(error["error"]["code"], "missing_capability");
    let message = error["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("automatic dependency repair is disabled"),
        "{message}"
    );
    assert!(!dir.path().join(".cerul-robotics/runtime/media").exists());
    assert!(!dir.path().join("sample.mp4.cerul").exists());
}
