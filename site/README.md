# Website

Static pages for the Cerul Robotics website: `index.html` (home),
`showcase.html` and `docs.html`. There is no framework, build step or
JavaScript. Fonts load from Google Fonts; everything else is in this directory.

Preview locally:

```sh
python3 -m http.server 4190 --directory site
```

Then open <http://localhost:4190>. Any static host can serve the directory
as-is.

## Editing

The pages reproduce the approved design. Styles are inline on purpose, so a
change is a direct edit of the element. Keep the three pages' navigation in
sync. The site has no sign-in, and the hosted API is described as not open.
Do not add either without a product decision.

Motion (playhead, label ticker, glow) is CSS-only and switches off under
`prefers-reduced-motion`. Hero label callouts are shown only at 1200 px and
wider.

## Media

`media/` holds showcase material the owner has cleared for publication:

| File | Source |
| --- | --- |
| `hands-*.jpg` | Frames of `review.mp4`, cropped to 1280×720 to remove the burned-in caption |
| `strip-*.jpg` | Frames of the original clip at 0.3–6.8 s, 320 px wide |
| `review.mp4` | `cerul-robotics render` output, re-encoded (H.264, CRF 27) for size |
| `annotations.json`, `summary.md` | `cerul-robotics annotate --semantic --hands` output |

The output came from `cerul-robotics` 0.1.0 with the default Gemini endpoint on
a 7-second first-person loom clip. Labels are unedited and not human-reviewed.
The only change to `annotations.json` is that the absolute local
`source.root` path is replaced with `.`. Add new showcase media only with
publication clearance.
