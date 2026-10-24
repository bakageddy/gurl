# gURL

A GUI for [HURL](https://hurl.dev), written in Tauri 2 (Rust) and Svelte 5. HURL runs in-process through the
`hurl` crates: nothing to install besides the app.

## Features

- **Workspace support**: open a folder and organise `.hurl` scenarios in sub-folders.
- **Variables per folder**: a `gurl.env` in any folder defines variables for its scenarios and every
  sub-folder; deeper folders override shallower ones. Secrets are redacted in logs and reports.
- **Visual builder**: methods, query/form/**multipart** (files from the scenario's folder), cookies, basic auth,
  JSON/XML/text/GraphQL/base64/hex/file bodies, per-request options, asserts (every query, filter and predicate),
  captures, and response status/version/headers and body assertions.
- **Text editor**: syntax highlighting and inline diagnostics, kept in sync with the visual builder.
- **Run** a request, a scenario, a folder or a whole workspace (parallel). Inspect bodies, headers, cookies,
  asserts, captures, a timing waterfall, cURL and TLS details.
- **Reports** in the app, run history, and export of HURL's own HTML / JUnit / TAP / JSON reports.

## Install

Download the latest build for your platform from the Releases page: `.dmg` (macOS, Apple Silicon and Intel),
NSIS `.exe` (Windows), `.deb`/`.AppImage` (Ubuntu 22.04 and 24.04), `.rpm` (Fedora), or the Arch tarball /
`packaging/arch/PKGBUILD`.

## Develop

```sh
cd gurl
npm install
npm run tauri dev
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup per OS, the project layout, checks and the release process.

## Credits

- The app icon lettering is set in [Commit Mono](https://commitmono.com) by Eigil Nikolajsen
  (SIL Open Font License 1.1).
- The UI uses [Inter](https://rsms.me/inter/) (OFL) and, off macOS, [DejaVu Sans Mono](https://dejavu-fonts.github.io/)
  for code; both are bundled. [HURL](https://hurl.dev) by Orange does the actual requests.
