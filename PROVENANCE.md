# Source provenance

Extracted from https://github.com/cerul-ai/cerul at commit `e93237887eaa1788c03b85e5e6220d674a1ed7d3`. Original history and release assets remain in that repository. Migrated Rust code is Apache-2.0; model and fixture notices retain their original licenses.

## Migration inventory

| Original Cerul paths | Robotics ownership |
| --- | --- |
| src/annotate/ | Semantic generation, hands, bundle export and review rendering |
| src/lerobot.rs and src/lerobot/ | Dataset reader, identity, transactional writer |
| models/hands/ | Embedded CPU models, hashes and licenses |
| prompts/{task,subtask,event,interaction,state,flag,progress,semantic-common}.md | Robotics model instructions |
| schemas/annotate-result.json, render-result.json, semantic domain schemas | Generated Robotics contracts |
| docs/annotation.md, lerobot*.md, design/annotation-experience.md | Product guides |
| examples/verify_lerobot_writeback.rs, tests/lerobot_roundtrip.py | Official loader acceptance harness |
| tests/fixtures/hand-opencv.png | Licensed real-inference fixture |
| Dataset discovery and cache tests in src/index/ | Cross-repository adapter regression tests |

The original source remains accessible at the source commit above. Shared
annotation records, hand frame data types, portable bundle data types and atomic
annotation layout remain in Cerul so existing data can be read without a Robotics
dependency. Inference, domain prompts and writeback are not part of core.

The CLI is separate. Its default registry/configuration uses `.cerul-robotics`;
project configuration is `cerul-robotics.toml`. Existing adjacent `.cerul` sidecars
retain their names for compatibility. An explicit workspace can reuse a previous
registry. Saved provider credentials may be read from Cerul without being changed.

Shared engine revision: `b92446d783df93bb97162e2ae807237814da7f9e`
([core extraction PR](https://github.com/cerul-ai/cerul/pull/292)).
New portable exports identify their generator as `cerul-robotics`; legacy Cerul
exports retain read compatibility.
