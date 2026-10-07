use crate::{config::Config, storage};
use cerul::index::{
    discover, embed,
    pipeline::{Options, run_with_adapter},
};
use tokio_util::sync::CancellationToken;
#[tokio::test]
async fn offline_pending_camera_does_not_block_cached_camera_rebuild() {
    use crate::index::vectors::{self, Kind, VectorRow};
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("dataset");
    crate::lerobot::tests::fixture(&root, "v3.1");
    let workspace = dir.path().join("workspace");
    let episode = crate::lerobot::read(&root).unwrap().remove(0);
    let sidecar = discover::publish_episode_with_adapter(
        &workspace,
        &episode,
        None,
        &crate::lerobot::Adapter,
    )
    .unwrap();
    let mut config = Config::default();
    config.embedding.base_url = "http://127.0.0.1:9".into();
    config.embedding.dims = Some(2);
    let options = Options {
        no_understanding: true,
        no_audio: true,
        no_ocr: true,
        streams: "all".into(),
        only: Some("0".into()),
        ..Default::default()
    };
    let space = config.space_id().unwrap();
    let stream = "observation.images.wrist";
    vectors::write(
        &sidecar.join("embeddings").join(format!("{space}.parquet")),
        &[VectorRow {
            id: "cached-wrist".into(),
            episode: episode.episode_id.clone(),
            stream: stream.into(),
            kind: Kind::Video,
            start_us: 0,
            end_us: 4_000_000,
            vector: vec![1., 0.],
            text: "cached".into(),
            still: false,
            space_id: space.clone(),
            params_hash: "fixture".into(),
        }],
        2,
    )
    .unwrap();
    storage::write_json(
        &embed::state_path(&sidecar, stream, &episode.time.reference, &space),
        &embed::State {
            input_hash: embed::fingerprint(
                &episode,
                stream,
                &space,
                &options.embedding,
                None,
                None,
            )
            .unwrap(),
            complete: true,
            error: None,
        },
    )
    .unwrap();
    assert!(!workspace.join("index").exists());
    let report = run_with_adapter(
        &[root],
        &workspace,
        &config,
        &options,
        CancellationToken::new(),
        &mut |_| {},
        &crate::lerobot::Adapter,
    )
    .await
    .unwrap();
    assert!(report.partial);
    let camera = report.episodes[0]
        .streams
        .iter()
        .find(|s| s.stream == stream)
        .unwrap();
    assert!(camera.indexed, "{:?}", camera.errors);
    assert_eq!(camera.vector_rows, 1);
    assert_eq!(
        cerul::index::lance::VectorIndex::open(&workspace, &space, 2, false)
            .await
            .unwrap()
            .count()
            .await
            .unwrap(),
        1
    );
}
