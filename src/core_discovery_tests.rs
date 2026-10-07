use crate::{
    episode::{Episode, Stream},
    media, storage,
};
use cerul::index::discover::*;
use std::{fs, path::Path};
fn publish_episode(w: &Path, e: &Episode, o: Option<&Path>) -> anyhow::Result<std::path::PathBuf> {
    publish_episode_with_adapter(w, e, o, &crate::lerobot::Adapter)
}
#[test]
fn tombstones_are_hidden_and_dataset_reconciliation_preserves_current_members() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("dataset");
    crate::lerobot::tests::fixture(&root, "v3.1");
    let workspace = dir.path().join("workspace");
    let mut episode = crate::lerobot::read_with_workspace(&root, &workspace)
        .unwrap()
        .remove(0);
    publish_episode(&workspace, &episode, None).unwrap();
    let kept = episode.episode_id.clone();
    episode.local_id = "999".into();
    episode.episode_id = format!("{}/999", episode.dataset_id);
    let removed = publish_episode(&workspace, &episode, None).unwrap();
    reconcile_dataset(
        &workspace,
        &root,
        &std::collections::BTreeSet::from([kept.clone()]),
    )
    .unwrap();
    assert_eq!(read_registry(&workspace).unwrap().len(), 1);
    assert!(removed.join("episode.json").exists());
    let mut entry = read_registry(&workspace).unwrap().remove(0);
    entry.pending_deletion = true;
    let sidecar = entry.sidecar.clone();
    register(&workspace, entry).unwrap();
    fs::remove_dir_all(sidecar).unwrap();
    assert!(read_registry(&workspace).unwrap().is_empty());
    assert_eq!(read_registry_all(&workspace).unwrap().len(), 1);
    assert!(
        crate::index::records::sidecars(&workspace)
            .unwrap()
            .is_empty()
    );
    reconcile_dataset(&workspace, &root, &std::collections::BTreeSet::new()).unwrap();
    assert_eq!(read_registry_all(&workspace).unwrap().len(), 1);
}

#[tokio::test]
async fn changed_dataset_publication_withdraws_only_affected_vectors_before_rebuild() {
    use cerul::index::{
        embed, lance,
        vectors::{self, Kind, VectorRow},
    };
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("dataset");
    crate::lerobot::tests::fixture(&root, "v3.1");
    let workspace = dir.path().join("workspace");
    let episode = crate::lerobot::read_with_workspace(&root, &workspace)
        .unwrap()
        .remove(0);
    let sidecar = publish_episode(&workspace, &episode, None).unwrap();
    let front = "observation.images.front";
    let wrist = "observation.images.wrist";
    let mut config = crate::config::Config::default();
    config.embedding.dims = Some(2);
    let space = config.space_id().unwrap();
    let rows = [front, wrist]
        .into_iter()
        .map(|stream| VectorRow {
            id: stream.into(),
            episode: episode.episode_id.clone(),
            stream: stream.into(),
            kind: Kind::Video,
            start_us: 0,
            end_us: 1_000_000,
            vector: vec![1., 0.],
            text: String::new(),
            still: false,
            space_id: space.clone(),
            params_hash: "fixture".into(),
        })
        .collect::<Vec<_>>();
    let parquet = sidecar.join("embeddings").join(format!("{space}.parquet"));
    vectors::write(&parquet, &rows, 2).unwrap();
    for stream in [front, wrist] {
        storage::write_json(
            &embed::state_path(&sidecar, stream, front, &space),
            &embed::State {
                input_hash: embed::fingerprint(
                    &episode,
                    stream,
                    &space,
                    &embed::Options::default(),
                    None,
                    None,
                )
                .unwrap(),
                complete: true,
                error: None,
            },
        )
        .unwrap();
    }
    let bytes = fs::read(&parquet).unwrap();
    publish_episode(&workspace, &episode, None).unwrap();
    assert!(embed::usable(&sidecar, front, front, &space).unwrap());
    assert_eq!(
        lance::rebuild(&workspace, &space, 2)
            .await
            .unwrap()
            .count()
            .await
            .unwrap(),
        2
    );
    let Stream::Video { path, .. } = episode.video(front).unwrap() else {
        unreachable!()
    };
    media::run(
        crate::media::command("ffmpeg")
            .args([
                "-v",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "color=c=blue:size=64x64:rate=2:duration=8",
                "-c:v",
                "libx264",
            ])
            .arg(root.join(path)),
    )
    .unwrap();
    let mut changed = crate::lerobot::read_with_workspace(&root, &workspace)
        .unwrap()
        .remove(0);
    assert_eq!(episode.episode_id, changed.episode_id);
    publish_episode(&workspace, &changed, None).unwrap();
    assert!(!embed::usable(&sidecar, front, front, &space).unwrap());
    assert!(embed::usable(&sidecar, wrist, front, &space).unwrap());
    assert_eq!(
        lance::rebuild(&workspace, &space, 2)
            .await
            .unwrap()
            .count()
            .await
            .unwrap(),
        1
    );
    let status = crate::status::inspect(&workspace, None).unwrap();
    assert!(
        !status.episodes[0]
            .embeddings
            .iter()
            .find(|state| state.stream == front)
            .unwrap()
            .complete
    );
    assert!(
        status.episodes[0]
            .embeddings
            .iter()
            .find(|state| state.stream == wrist)
            .unwrap()
            .complete
    );
    for filter in [format!("stream={front}"), format!("stream!={wrist}")] {
        let error = crate::search::run(
            &workspace,
            &config,
            &crate::search::Options {
                query: Some("cup".into()),
                filters: vec![filter],
                ..Default::default()
            },
            tokio_util::sync::CancellationToken::new(),
        )
        .await
        .unwrap_err();
        assert_eq!(
            error
                .downcast_ref::<crate::providers::ProviderError>()
                .unwrap()
                .kind,
            crate::providers::Failure::Unsupported
        );
    }
    // An unrelated camera mapping must not invalidate this stream's products.
    let previous_keys = source_keys(&changed).unwrap();
    let front_fingerprint = embed::fingerprint(
        &changed,
        front,
        &space,
        &embed::Options::default(),
        None,
        None,
    )
    .unwrap();
    changed.time.mappings.get_mut(wrist).unwrap().b_us += 1;
    let changed_keys = source_keys(&changed).unwrap();
    assert_eq!(previous_keys[front], changed_keys[front]);
    assert_ne!(previous_keys[wrist], changed_keys[wrist]);
    assert_eq!(
        front_fingerprint,
        embed::fingerprint(
            &changed,
            front,
            &space,
            &embed::Options::default(),
            None,
            None,
        )
        .unwrap()
    );
    publish_episode(&workspace, &changed, None).unwrap();
    assert!(!embed::usable(&sidecar, wrist, front, &space).unwrap());
    assert_eq!(
        lance::rebuild(&workspace, &space, 2)
            .await
            .unwrap()
            .count()
            .await
            .unwrap(),
        0
    );
    let error = crate::search::run(
        &workspace,
        &config,
        &crate::search::Options {
            query: Some("cup".into()),
            ..Default::default()
        },
        tokio_util::sync::CancellationToken::new(),
    )
    .await
    .unwrap_err();
    assert_eq!(
        error
            .downcast_ref::<crate::providers::ProviderError>()
            .unwrap()
            .kind,
        crate::providers::Failure::Unsupported
    );
    assert!(
        crate::status::inspect(&workspace, None).unwrap().episodes[0]
            .embedding_spaces
            .is_empty()
    );
    assert_eq!(fs::read(parquet).unwrap(), bytes);
}
#[tokio::test]
async fn reference_duration_growth_invalidates_newly_exposed_secondary_coverage() {
    use cerul::index::{
        embed, lance,
        vectors::{self, Kind, VectorRow},
    };
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("dataset");
    crate::lerobot::tests::fixture(&root, "v3.1");
    let workspace = dir.path().join("workspace");
    let extended = crate::lerobot::read_with_workspace(&root, &workspace)
        .unwrap()
        .remove(0);
    let front = "observation.images.front";
    let wrist = "observation.images.wrist";
    let mut shorter = extended.clone();
    for stream in &mut shorter.streams {
        if let Stream::Video { id, range_us, .. } = stream
            && id == front
        {
            range_us[1] = range_us[0] + 2_000_000;
        }
    }
    assert_eq!(
        shorter.video_coverage(wrist).unwrap().unwrap().end_us,
        2_000_000
    );
    assert_eq!(
        extended.video_coverage(wrist).unwrap().unwrap().end_us,
        4_000_000
    );
    let sidecar = publish_episode(&workspace, &shorter, None).unwrap();
    let space = "a".repeat(64);
    let parquet = sidecar.join("embeddings").join(format!("{space}.parquet"));
    vectors::write(
        &parquet,
        &[VectorRow {
            id: "wrist".into(),
            episode: shorter.episode_id.clone(),
            stream: wrist.into(),
            kind: Kind::Video,
            start_us: 0,
            end_us: 2_000_000,
            vector: vec![1., 0.],
            text: String::new(),
            still: false,
            space_id: space.clone(),
            params_hash: "fixture".into(),
        }],
        2,
    )
    .unwrap();
    storage::write_json(
        &embed::state_path(&sidecar, wrist, front, &space),
        &embed::State {
            input_hash: embed::fingerprint(
                &shorter,
                wrist,
                &space,
                &embed::Options::default(),
                None,
                None,
            )
            .unwrap(),
            complete: true,
            error: None,
        },
    )
    .unwrap();
    let bytes = fs::read(&parquet).unwrap();
    let params = serde_json::json!({});
    assert_ne!(
        cerul::index::stations::station_key(&shorter, wrist, "semantic.subtask", &params).unwrap(),
        cerul::index::stations::station_key(&extended, wrist, "semantic.subtask", &params).unwrap()
    );
    assert!(embed::usable(&sidecar, wrist, front, &space).unwrap());
    publish_episode(&workspace, &extended, None).unwrap();
    assert!(!embed::usable(&sidecar, wrist, front, &space).unwrap());
    assert_eq!(
        lance::rebuild(&workspace, &space, 2)
            .await
            .unwrap()
            .count()
            .await
            .unwrap(),
        0
    );
    assert_eq!(fs::read(&parquet).unwrap(), bytes);
}
