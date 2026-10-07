# Robotics behavioral contracts

Extracted from Cerul DESIGN.md at the source revision in PROVENANCE.md. Commands below use the `cerul-robotics` binary.

### `annotate <path>... --semantic [items] --hands`

| Options | Behavior/default |
| --- | --- |
| `--semantic` | Optional explicit types; default subtask, event, interaction, state for every input format. |
| `--embodied` | Hidden compatibility no-op; annotation is embodied-only. General-mode generation is removed; old records remain readable. |
| `--hands` | Optional local human-hand keypoints. Never enabled automatically. `--semantic none` makes this an offline hand-only run. |
| `--grounding` | Generic selector remains reserved; use `--hands` for the implemented local subtype. |
| `--world` | Reserved; rejected as unsupported. |
| `--streams`, `--only`, `--ontology FILE`, `--window 30s`, `--fps 2` | Verbs are unrestricted unless an ontology is supplied, regardless of input format. |
| `--write-lerobot`, `--out DIR` | No dataset mutation by default. Writeback accepts only an existing compatible v3.1 dataset. |

Each semantic subtype is a module: frames → timestamped contact sheet → schema-constrained model response → staging → validation → annotation publication → records index update. Invalid modules publish nothing; independent modules can still succeed.

### `render <video-or-annotations.json> --out FILE`

Render published semantic labels and human-hand skeletons into a new MP4 with no model calls. Hand coordinates are applied to the original display image; no missing detections are interpolated. The caption
panel preserves the source image; source timestamps and audio are rebased by the
same clip origin. `--stream` selects a camera from a dataset bundle; non-unit
time scaling is rejected. The output records Cerul version/generation metadata;
`--watermark` opts into a visible Cerul signature. Existing outputs are never
overwritten. Source hashes and per-track provenance are validated before rendering.

Annotation completion publishes a portable `annotations.json` and readable
`summary.md` per episode. The bundle includes source identity, a single Cerul
generator marker and full per-track provenance. Both views carry a content
generation; each is atomically replaced and a rerun repairs interrupted pairs.
Sidecars remain authoritative. `annotation_progress` reports planned work,
completed units, cached units and phase; success is only complete after export
and record-index publication.

### Semantic annotation

Sample at --fps (default 2), use a five-column contact grid, and burn **clip-relative** timestamps into frames. Each window starts at zero. Add clip_start_us exactly once to returned model times.

Subtask annotation describes first, then segments. Define boundaries using holding, release, arrival, and state change. Windows overlap by five seconds; conflicting outputs produce flags. Snap boundaries to real frame PTS, require continuous subtask coverage, validate ontology verbs where applicable, and keep normalized coordinates within [0,1].

### LeRobot writeback

The CLI writes only subtask language entries to an already compatible v3.1 dataset. v3.0 returns code 3 and directs users to official format tooling; Cerul performs no upgrade. The pinned upstream recorder/compatibility-fixture distinction is documented in ../lerobot.md and must not be misrepresented as an available official upgrade command.

Use language_persistent with style=subtask, exactly one active subtask per frame, and timestamps from the source Parquet frame values without recalculation. Preserve action, state, tasks, language_events, and all non-subtask language_persistent entries. Add or replace only subtask entries. Retain info.json content, updating only necessary language feature metadata when adding a column. Events/flags stay in sidecars because they have no defined official style.

Prefer --out. Stage a complete dataset, perform native field/timeline validation, then publish. Release acceptance additionally uses the official Python loader to read all sample frames; Python is an acceptance dependency, not a runtime dependency. In-place writeback requires recoverable file replacement records and must preserve unselected episodes in shared shards. Compare all affected shards' protected fields and existing annotations, not just one sampled frame.

