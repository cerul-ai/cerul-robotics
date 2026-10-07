use anyhow::Result;
use schemars::{JsonSchema, Schema};
use std::{fs, path::Path};
fn schema<T: JsonSchema>() -> Schema {
    schemars::schema_for!(T)
}
fn main() -> Result<()> {
    let check = std::env::args().any(|argument| argument == "--check");
    let schemas = [
        (
            "annotations",
            schema::<cerul::annotations::portable::Bundle>(),
        ),
        (
            "grounding-hand-frame",
            schema::<cerul::annotations::hand::Frame>(),
        ),
        (
            "annotate-result",
            schema::<cerul_robotics::annotate::pipeline::Report>(),
        ),
        (
            "render-result",
            schema::<cerul_robotics::annotate::video::Report>(),
        ),
        (
            "semantic-task",
            schema::<
                cerul_robotics::annotate::schema::Window<cerul_robotics::annotate::schema::Task>,
            >(),
        ),
        (
            "semantic-subtask",
            schema::<
                cerul_robotics::annotate::schema::Window<cerul_robotics::annotate::schema::Subtask>,
            >(),
        ),
        (
            "semantic-event",
            schema::<
                cerul_robotics::annotate::schema::Window<cerul_robotics::annotate::schema::Event>,
            >(),
        ),
        (
            "semantic-interaction",
            schema::<
                cerul_robotics::annotate::schema::Window<
                    cerul_robotics::annotate::schema::Interaction,
                >,
            >(),
        ),
        (
            "semantic-state",
            schema::<
                cerul_robotics::annotate::schema::Window<cerul_robotics::annotate::schema::State>,
            >(),
        ),
        (
            "semantic-flag",
            schema::<
                cerul_robotics::annotate::schema::Window<cerul_robotics::annotate::schema::Flag>,
            >(),
        ),
        (
            "semantic-progress",
            schema::<
                cerul_robotics::annotate::schema::Window<
                    cerul_robotics::annotate::schema::Progress,
                >,
            >(),
        ),
    ];
    if !check {
        fs::create_dir_all("schemas")?;
    }
    for (name, mut schema) in schemas {
        schema.insert(
            "$id".into(),
            serde_json::Value::String(format!("https://cerul.ai/schemas/{name}/1")),
        );
        let path = Path::new("schemas").join(format!("{name}.json"));
        let expected = format!("{}\n", serde_json::to_string_pretty(&schema)?);
        if check {
            anyhow::ensure!(
                fs::read_to_string(&path)? == expected,
                "stale schema {}",
                path.display()
            );
        } else {
            fs::write(path, expected)?;
        }
    }
    Ok(())
}
