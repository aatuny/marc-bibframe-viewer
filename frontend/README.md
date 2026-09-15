# frontend

Astro frontend for displaying both MARC and BIBFRAME within a browser.

This is a minimal placeholder project just so that the tool may be locally used.

## Development

To start the development server:

1. Install dependencies using: `npm i`
2. Start server using: `npm run dev`
3. Browse to `localhost:4321`

Note that you must have built an index over the source XML file beforehand and the API provided in `crates/bf-viewer-api`.

Requests to `/api` are proxied to `127.0.0.1:8080` by the dev server. If the API listens elsewhere, adjust the proxy target in `astro.config.mjs`.

## Notes

Desktop only: below 1024px the viewer is hidden and a notice shown.

Astro's scoped styles do not reach JS-built elements, so the tree rules use `:global()` behind a container class.

## Use of AI disclosure

In addition to the main README information it's worth noting that use of AI assistance (Claude Code) shows probably the most in CSS and in the custom way of rendering the triplets.

## License

MIT
