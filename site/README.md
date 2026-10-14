# Website

Static pages for https://robotics.cerul.ai: `index.html` (home),
`showcase.html`, `docs.html` and `404.html`. There is no framework or build
step. Shared behaviour lives in `assets/site.css` and `assets/site.js`; fonts
load from Google Fonts; everything else is in this directory.

The site is a Cloudflare Worker with static assets only (`wrangler.jsonc` at
the repository root). Pages are served at clean URLs (`/showcase`, `/docs`),
`_headers` sets caching and security headers, and `.assetsignore` keeps this
README unpublished. Preview locally with the same routing:

```sh
npx wrangler@4.136.0 dev
```

Deployment is Cloudflare Workers Builds, connected to this repository in the
Cloudflare dashboard (Worker `cerul-robotics-site`). A push to `main` that
touches `site/` or `wrangler.jsonc` runs `npx wrangler@4.136.0 deploy`; there
is no build step and no other branch builds. No Cloudflare credential is
stored in GitHub. To deploy by hand, run the same command with a logged-in
Wrangler.

Search and sharing: every page has a canonical URL and Open Graph/Twitter
tags pointing at `og.jpg`; the home page carries `SoftwareApplication`
structured data. Keep `sitemap.xml` and `llms.txt` in step with the pages.

## Editing

The pages started from the approved design and are now the source of truth.
Layout styles are inline, so a change is a direct edit of the element. Keep
the three pages' navigation and logo in sync. The site has no sign-in, and the
hosted API is described as not open. Do not add either without a product
decision. The logo is the Cerul mark from the brand master with the Instrument
Serif wordmark; the favicon is the brand's adaptive icon.

`site.js` drives one clock per `[data-episode]` block. With a video inside, the
playhead, active lanes (`[data-s][data-e]`), label rows and hero callouts
follow `video.currentTime`; the label data mirrors `media/summary.md`.

Motion follows the brand rules: an on-page pause control (stored per
browser), no playback while offscreen or while the tab is hidden, and a static
fallback under `prefers-reduced-motion` for decorative motion (ticker, scan
light, push-in, reveals, count-ups). The muted demo video is content and still
plays under reduced motion, stopping with the pause control. Hero callouts are
shown only at 1200 px and wider.

## Media

`media/` holds showcase material the owner has cleared for publication:

| File | Source |
| --- | --- |
| `hands-*.jpg` | Frames of `review.mp4`, cropped to 1280×720 to remove the burned-in caption |
| `strip-*.jpg` | Frames of the original clip at 0.3–6.8 s, 320 px wide |
| `review.mp4` | `cerul-robotics render` output, re-encoded (H.264, CRF 27) for size |
| `hero.mp4` | The same render cropped to 1280×720 to remove the caption, H.264 CRF 31, for the looping hero and showcase stage |
| `annotations.json`, `summary.md` | `cerul-robotics annotate --semantic --hands` output |
| `../og.jpg` | 1200×630 share image: a `hands-3.9.jpg` frame with the logo and episode lanes |

The output came from `cerul-robotics` 0.1.0 with the default Gemini endpoint on
a 7-second first-person loom clip. Labels are unedited and not human-reviewed.
The only change to `annotations.json` is that the absolute local
`source.root` path is replaced with `.`. Add new showcase media only with
publication clearance.
