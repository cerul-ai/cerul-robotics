use anyhow::{Context, Result, ensure};
use cerul::{
    config::Config,
    events::Event,
    providers::{Failure, ProviderError},
};
use cerul_robotics::{annotate, lerobot};
use clap::{Args, Parser, Subcommand};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tokio_util::sync::CancellationToken;
const ANNOTATE_EXAMPLES: &str = "\
Examples:
  cerul-robotics annotate ./video.mp4 --semantic
      Label embodied steps, events, interactions, and states (no index step needed).
  cerul-robotics annotate ./video.mp4 --semantic subtask,event,interaction,state
      Label action steps, events, contacts, and state changes in a demonstration.
  cerul-robotics annotate ./dataset --semantic --only 0
      Label the first LeRobot episode with embodied defaults.
  cerul-robotics annotate ./video.mp4 --semantic --dry-run
      Preview the work without writing files or calling models.
  cerul-robotics annotate ./video.mp4 --hands --semantic none
      Track human hands locally on CPU, with no model key required.

Types: task, subtask, event, interaction, state, flag, progress.
Defaults: subtask,event,interaction,state. Annotation is embodied-only; use analyze for general videos.
Outputs: annotations.json and summary.md; internal semantic.<type>.jsonl sidecars retain provenance.
LeRobot: annotations live in .cerul/episodes/<episode_index>/ inside the dataset.
--out is a new LeRobot dataset copy and requires --write-lerobot; it is not a
JSONL export directory. See the LeRobot guide for supported writeback versions.

Semantic labels use your configured vision endpoint (Gemini by default): cerul auth set.
--hands is optional and never enabled automatically. Add --semantic none for hands only.
Depth, calibrated 3D poses, and robot gripper detection are not supported.
Guide: https://github.com/cerul-ai/cerul-robotics/blob/main/docs/annotation.md";
const ADVANCED: &str = "Advanced";

#[derive(Parser)]
#[command(
    name = "cerul-robotics",
    version,
    about = "Search, annotate and review robot episodes with Cerul"
)]
struct Cli {
    #[arg(long, global = true, env = "CERUL_ROBOTICS_WORKSPACE")]
    workspace: Option<PathBuf>,
    /// Check media tools without downloading or selecting automatic repairs
    #[arg(long, global = true)]
    no_auto_deps: bool,
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, global = true)]
    dry_run: bool,
    #[arg(long, global = true)]
    recompute: bool,
    #[arg(long = "set", global = true)]
    overrides: Vec<String>,
    /// Suppress the provider media-transfer notice.
    #[arg(short = 'y', long, global = true)]
    yes: bool,
    #[arg(short = 'q', long, global = true)]
    quiet: bool,
    #[arg(short = 'v', long, global = true)]
    verbose: bool,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Generate semantic labels and optional local human-hand keypoints.
    #[command(arg_required_else_help=true, before_help=ANNOTATE_EXAMPLES)]
    Annotate(AnnotateArgs),
    /// Render published annotations with no model calls; never overwrite output.
    Render {
        path: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        stream: Option<String>,
        #[arg(long)]
        watermark: bool,
    },
    /// Index ordinary videos or LeRobot datasets through the shared engine.
    Index(InputArgs),
    /// Analyze scenes and summaries with the shared engine.
    Analyze(AnalyzeArgs),
    /// Search the shared workspace and optionally export matched clips.
    Search {
        query: Option<String>,
        #[arg(long = "filter")]
        filters: Vec<String>,
        #[arg(long = "in")]
        within: Option<PathBuf>,
        #[arg(long)]
        text: bool,
        #[arg(long)]
        count: bool,
        #[arg(long)]
        save: Option<PathBuf>,
        #[arg(long, default_value_t = 10)]
        limit: usize,
    },
    /// Inspect registered results without contacting a model.
    Status {
        path: Option<PathBuf>,
        #[arg(long, requires = "path")]
        timeline: bool,
        #[arg(long = "type", requires = "timeline")]
        kind: Option<String>,
        #[arg(long, requires = "timeline")]
        limit: Option<usize>,
    },
    /// Print the Robotics agent skill.
    Skill,
}
#[derive(Args)]
struct InputArgs {
    #[arg(required = true)]
    paths: Vec<PathBuf>,
    #[arg(long, default_value = "primary")]
    streams: String,
    #[arg(long)]
    only: Option<String>,
    #[arg(long, default_value_t = 4)]
    jobs: usize,
    #[arg(long)]
    rpm: Option<u32>,
    #[arg(long)]
    no_audio: bool,
    #[arg(long)]
    no_ocr: bool,
}
#[derive(Args)]
struct AnalyzeArgs {
    #[arg(required = true)]
    paths: Vec<PathBuf>,
    #[arg(long, default_value = "primary")]
    streams: String,
    #[arg(long)]
    only: Option<String>,
    #[arg(long, default_value_t = 4)]
    jobs: usize,
    #[arg(long)]
    rpm: Option<u32>,
}
#[derive(Args)]
struct AnnotateArgs {
    /// Videos, directories, or LeRobot datasets.
    #[arg(required = true)]
    paths: Vec<PathBuf>,
    /// Semantic items, comma-separated; use none with --hands for offline hand annotation.
    #[arg(long,num_args=0..=1,default_missing_value="default", value_name = "ITEMS")]
    semantic: Option<String>,
    /// Compatibility flag: annotation is always embodied.
    #[arg(long, hide = true, default_value_t = true)]
    embodied: bool,
    /// Add local human-hand keypoints to an embodied demonstration (CPU, no API calls).
    #[arg(long)]
    hands: bool,
    /// Write subtask annotations back into the LeRobot dataset.
    #[arg(long)]
    write_lerobot: bool,
    /// New output LeRobot dataset (requires --write-lerobot).
    #[arg(long, value_name = "DIR")]
    out: Option<PathBuf>,
    /// Custom ontology file.
    #[arg(long, value_name = "FILE", help_heading = ADVANCED)]
    ontology: Option<PathBuf>,
    /// Model window length, for example 30s.
    #[arg(long,default_value="30s",value_parser=duration, help_heading = ADVANCED)]
    window: i64,
    /// Frames per second sampled for the model.
    #[arg(long, default_value_t = 2., help_heading = ADVANCED)]
    fps: f64,
    /// Parallel model requests.
    #[arg(long, default_value_t = 4, help_heading = ADVANCED)]
    jobs: usize,
    /// Cap on model requests per minute.
    #[arg(long, help_heading = ADVANCED)]
    rpm: Option<u32>,
    /// Streams to annotate, comma-separated.
    #[arg(long, default_value = "primary", help_heading = ADVANCED)]
    streams: String,
    /// Only these episodes (ids or local indexes), comma-separated.
    #[arg(long, help_heading = ADVANCED)]
    only: Option<String>,
    #[arg(long,num_args=0..=1,default_missing_value="default", hide = true)]
    grounding: Option<String>,
    #[arg(long,num_args=0..=1,default_missing_value="default", hide = true)]
    world: Option<String>,
}
fn duration(raw: &str) -> std::result::Result<i64, String> {
    let (number, factor) = if let Some(value) = raw.strip_suffix("ms") {
        (value, 1_000i64)
    } else if let Some(value) = raw.strip_suffix("us") {
        (value, 1i64)
    } else if let Some(value) = raw.strip_suffix('s') {
        (value, 1_000_000i64)
    } else if let Some(value) = raw.strip_suffix('m') {
        (value, 60_000_000i64)
    } else {
        return Err("duration requires us, ms, s, or m".into());
    };
    let value = number.parse::<f64>().map_err(|_| "invalid duration")? * factor as f64;
    if !value.is_finite()
        || value < 0.
        || value >= i64::MAX as f64
        || value.fract().abs() > 0.000001
    {
        return Err("duration must be nonnegative whole microseconds".into());
    }
    Ok(value.round() as i64)
}
fn overrides(cli: &Cli) -> Result<toml::Table> {
    let mut overlay = toml::Table::new();
    for item in &cli.overrides {
        let (key, value) = item
            .split_once('=')
            .context("--set requires KEY=TOML_VALUE")?;
        let (section, field) = key
            .split_once('.')
            .context("--set key requires section.field")?;
        anyhow::ensure!(!field.contains('.'), "--set supports endpoint fields only");
        let parsed: toml::Table = toml::from_str(&format!("value={value}"))?;
        let section = overlay
            .entry(section.to_owned())
            .or_insert_with(|| toml::Value::Table(Default::default()));
        section
            .as_table_mut()
            .unwrap()
            .insert(field.to_owned(), parsed["value"].clone());
    }
    Ok(overlay)
}

fn rate_limited(message: &str) -> bool {
    let s = message.to_ascii_lowercase();
    s.contains("429") || s.contains("rate limit") || s.contains("resource_exhausted")
}
fn retry_after(
    report: &cerul_robotics::annotate::pipeline::Report,
    original: &[String],
) -> Option<cerul_robotics::annotate::pipeline::Retry> {
    if !report.partial || report.dry_run {
        return None;
    }
    let limited = report
        .modules
        .iter()
        .filter_map(|module| module.error.as_deref())
        .any(rate_limited);
    // Keeping the program exactly as it was invoked: a path was probably used
    // because the binary is not on PATH, and shortening it would break the copy.
    let (program, rest) = original.split_first()?;
    let mut argv = vec![program.clone()];
    // The rate cap is the one argument this command is allowed to replace.
    let mut rpm: Option<u32> = None;
    let mut expecting = false;
    for argument in rest {
        if expecting {
            expecting = false;
            rpm = argument.parse().ok();
            continue;
        }
        if argument == "--rpm" {
            expecting = true;
            continue;
        }
        if let Some(value) = argument.strip_prefix("--rpm=") {
            rpm = value.parse().ok();
            continue;
        }
        argv.push(argument.clone());
    }
    let reason = match limited {
        true => cerul_robotics::annotate::pipeline::RetryReason::RateLimit,
        false => cerul_robotics::annotate::pipeline::RetryReason::Incomplete,
    };
    // Halving a cap the person already chose respects their intent; a first
    // limit starts low enough that a retry is worth attempting at all.
    let next = match (limited, rpm) {
        (true, Some(current)) => Some((current / 2).max(1)),
        (true, None) => Some(6),
        (false, current) => current,
    };
    if let Some(value) = next {
        argv.push("--rpm".into());
        argv.push(value.to_string());
    }
    Some(cerul_robotics::annotate::pipeline::Retry { argv, reason })
}

#[derive(Debug)]
struct ConfigurationError(String);
impl std::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ConfigurationError {}
fn invalid(error: anyhow::Error) -> anyhow::Error {
    ConfigurationError(format!("{error:#}")).into()
}
fn credentials() -> cerul::providers::CredentialResolver {
    Arc::new(|endpoint| {
        Box::pin(async move {
            let Some(home) = std::env::var_os("HOME") else {
                return Ok(None);
            };
            let path = PathBuf::from(home).join(".cerul/credentials.json");
            let metadata = match std::fs::symlink_metadata(&path) {
                Ok(m) => m,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(e.into()),
            };
            ensure!(metadata.is_file(), "credential file must be a regular file");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                ensure!(
                    metadata.permissions().mode() & 0o077 == 0,
                    "credential file must be private: chmod 600 ~/.cerul/credentials.json"
                );
            }
            let keys: BTreeMap<String, String> = serde_json::from_slice(&std::fs::read(path)?)?;
            Ok(keys
                .get(&cerul::providers::credential_scope(&endpoint))
                .cloned())
        })
    })
}
/// Rejects unreadable inputs before any media preparation, plan or model request.
fn readable(paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        ensure!(
            path.exists(),
            ConfigurationError(format!("no such file or directory: {}", path.display()))
        );
    }
    Ok(())
}

/// Same rule as the core CLI: verify or fetch compatible media tools before a
/// command decodes or encodes media, so a fresh install needs no system FFmpeg.
async fn prepare_media(
    cli: &Cli,
    workspace: &std::path::Path,
    cancel: &CancellationToken,
    events: &mut dyn cerul::events::EventSink,
) -> Result<()> {
    cerul::media::dependencies::prepare(workspace, !cli.no_auto_deps, cli.dry_run, cancel, events)
        .await
        .map_err(|error| {
            ProviderError {
                kind: Failure::Unsupported,
                message: format!("{error:#}"),
            }
            .into()
        })
}
async fn run(cli: &Cli, cancel: CancellationToken) -> Result<(Value, u8)> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let workspace = cli
        .workspace
        .clone()
        .or_else(|| home.as_ref().map(|p| p.join(".cerul-robotics")))
        .context("set --workspace when HOME is unavailable")?;
    let mut paths = Vec::new();
    if let Some(home) = home {
        paths.push(home.join(".cerul-robotics/config.toml"));
    }
    paths.push(std::env::current_dir()?.join("cerul-robotics.toml"));
    let environment = std::env::vars()
        .filter(|(k, _)| k.starts_with("CERUL_"))
        .collect();
    let mut config = Config::resolve(
        &paths,
        &environment,
        toml::Value::Table(overrides(cli).map_err(invalid)?),
    )
    .map_err(invalid)?;
    // Auto-enable speech only when an environment credential is already available.
    if config.transcription.enabled.is_none() {
        config.transcription.enabled = Some(
            std::env::var(&config.transcription.api_key_env).is_ok_and(|k| !k.trim().is_empty()),
        );
    }
    let mut events = |event: Event| {
        if cli.json {
            eprintln!(
                "{}",
                serde_json::to_string(&event).expect("serializable event")
            );
        } else if cli.verbose && !cli.quiet {
            eprintln!("{event:?}");
        }
    };
    let json_mode = cli.json;
    let seen = std::sync::Mutex::new(std::collections::BTreeSet::new());
    let notice: Option<cerul::providers::RequestNotice> = (!cli.yes && !cli.quiet).then(|| {
        Arc::new(move |endpoint: &cerul::config::Endpoint| {
            if !seen
                .lock()
                .unwrap()
                .insert(cerul::providers::credential_scope(endpoint))
            {
                return;
            }
            let msg = format!(
                "Model processing sends sampled media to {} ({}).",
                endpoint.base_url, endpoint.model
            );
            if json_mode {
                eprintln!(
                    "{}",
                    serde_json::to_string(&Event::Log {
                        level: "info".into(),
                        msg
                    })
                    .unwrap()
                );
            } else {
                eprintln!("{msg}");
            }
        }) as cerul::providers::RequestNotice
    });
    match &cli.command {
        Command::Annotate(a) => {
            if a.grounding.is_some() || a.world.is_some() {
                return Err(ProviderError {
                    kind: Failure::Unsupported,
                    message: "grounding and world annotations are not implemented".into(),
                }
                .into());
            }
            let options = annotate::pipeline::Options {
                items: match a.semantic.as_deref() {
                    None | Some("default" | "none") => vec![],
                    Some(v) => v.split(',').map(str::to_owned).collect(),
                },
                embodied: a.embodied,
                hands: a.hands,
                no_semantic: a.semantic.as_deref() == Some("none"),
                write_lerobot: a.write_lerobot,
                out: a.out.clone(),
                streams: a.streams.clone(),
                only: a.only.clone(),
                ontology: a.ontology.clone(),
                window_us: a.window,
                fps: a.fps,
                recompute: cli.recompute,
                dry_run: cli.dry_run,
                jobs: a.jobs,
                rpm: a.rpm,
                request_notice: notice,
            };
            options.validate().map_err(invalid)?;
            readable(&a.paths)?;
            prepare_media(cli, &workspace, &cancel, &mut events).await?;
            let mut report = annotate::pipeline::run(
                &a.paths,
                &workspace,
                &config,
                &options,
                cancel,
                &mut events,
            )
            .await?;
            report.retry = retry_after(&report, &std::env::args().collect::<Vec<_>>());
            let code = if report.partial { 6 } else { 0 };
            Ok((serde_json::to_value(report)?, code))
        }
        Command::Render {
            path,
            out,
            stream,
            watermark,
        } => {
            readable(std::slice::from_ref(path))?;
            prepare_media(cli, &workspace, &cancel, &mut events).await?;
            Ok((
                serde_json::to_value(annotate::video::render(
                    path,
                    &workspace,
                    out,
                    stream.as_deref(),
                    *watermark,
                    cli.dry_run,
                    &cancel,
                )?)?,
                0,
            ))
        }
        Command::Index(a) => {
            ensure!(
                a.jobs > 0 && a.rpm != Some(0),
                ConfigurationError("jobs and RPM must be positive".into())
            );
            let mut options = cerul::index::pipeline::Options {
                no_audio: a.no_audio,
                no_ocr: a.no_ocr,
                no_understanding: true,
                streams: a.streams.clone(),
                only: a.only.clone(),
                jobs: a.jobs,
                rpm: a.rpm,
                dry_run: cli.dry_run,
                request_notice: notice,
                ..Default::default()
            };
            options.embedding.recompute = cli.recompute;
            readable(&a.paths)?;
            prepare_media(cli, &workspace, &cancel, &mut events).await?;
            let report = cerul::index::pipeline::run_with_adapter(
                &a.paths,
                &workspace,
                &config,
                &options,
                cancel,
                &mut events,
                &lerobot::Adapter,
            )
            .await?;
            let code = if report.partial { 6 } else { 0 };
            Ok((serde_json::to_value(report)?, code))
        }
        Command::Analyze(a) => {
            ensure!(
                a.jobs > 0 && a.rpm != Some(0),
                ConfigurationError("jobs and RPM must be positive".into())
            );
            let options = cerul::analyze::Options {
                streams: a.streams.clone(),
                only: a.only.clone(),
                jobs: a.jobs,
                rpm: a.rpm,
                recompute: cli.recompute,
                dry_run: cli.dry_run,
                request_notice: notice,
                ..Default::default()
            };
            readable(&a.paths)?;
            prepare_media(cli, &workspace, &cancel, &mut events).await?;
            let report = cerul::analyze::run_with_adapter(
                &a.paths,
                &workspace,
                &config,
                &options,
                cancel,
                &mut events,
                &lerobot::Adapter,
            )
            .await?;
            let code = if report.partial { 6 } else { 0 };
            Ok((serde_json::to_value(report)?, code))
        }
        Command::Search {
            query,
            filters,
            within,
            text,
            count,
            save,
            limit,
        } => {
            let options = cerul::search::Options {
                query: query.clone(),
                filters: filters.clone(),
                within: within.clone(),
                text: *text,
                count: *count,
                save: save.clone(),
                limit: *limit,
                dry_run: cli.dry_run,
                request_notice: notice,
                ..Default::default()
            };
            options.validate().map_err(invalid)?;
            if options.save.is_some() && !cli.dry_run {
                prepare_media(cli, &workspace, &cancel, &mut events).await?;
            }
            Ok((
                serde_json::to_value(
                    cerul::search::run(&workspace, &config, &options, cancel).await?,
                )?,
                0,
            ))
        }
        Command::Status {
            path,
            timeline,
            kind,
            limit,
        } => {
            let value = if *timeline {
                serde_json::to_value(cerul::status::timeline(
                    &workspace,
                    path.as_deref(),
                    &cerul::status::TimelineOptions {
                        kind: kind.clone(),
                        limit: limit.unwrap_or(50),
                    },
                )?)?
            } else {
                serde_json::to_value(cerul::status::inspect(&workspace, path.as_deref())?)?
            };
            Ok((value, 0))
        }
        Command::Skill => unreachable!(),
    }
}
#[tokio::main]
async fn main() -> std::process::ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let code = error.exit_code() as u8;
            if code != 0 && std::env::args().any(|a| a == "--json") {
                println!(
                    "{}",
                    json!({"error":{"code":"invalid_arguments","message":error.to_string()}})
                );
            } else {
                let _ = error.print();
            }
            return code.into();
        }
    };
    if matches!(cli.command, Command::Skill) {
        print!("{}", include_str!("../skills/cerul-robotics/SKILL.md"));
        return std::process::ExitCode::SUCCESS;
    }
    let cancel = CancellationToken::new();
    let signal = cancel.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            signal.cancel();
        }
    });
    let result = cerul::providers::with_credential_resolver(
        credentials(),
        cerul::media::with_cancellation(cancel.clone(), run(&cli, cancel)),
    )
    .await;
    match result {
        Ok((value, code)) => {
            if cli.json {
                println!("{value}");
            } else {
                println!("{}", serde_json::to_string_pretty(&value).unwrap());
            }
            code.into()
        }
        Err(error) => {
            let code = if error.is::<ConfigurationError>() {
                2
            } else {
                match error.downcast_ref::<ProviderError>().map(|e| e.kind) {
                    Some(Failure::Cancelled) => 5,
                    Some(Failure::Unsupported) => 3,
                    _ => 4,
                }
            };
            if cli.json {
                println!(
                    "{}",
                    json!({"error":{"message":format!("{error:#}"),"code":match code {2=>"invalid_configuration",3=>"missing_capability",5=>"cancelled",_=>"execution_failed"}}})
                );
            } else {
                eprintln!("{error:#}");
            }
            code.into()
        }
    }
}
