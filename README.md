# gURL

A native-feeling GUI for [HURL](https://hurl.dev), built with Tauri 2 (Rust) and Svelte 5. HURL runs
in-process through the `hurl` crates: nothing to install besides the app.

## Features

- **Workspaces**: open a folder; nest up to 255 levels of sub-folders, each with many `.hurl` scenarios.
- **Variables per folder**: a `gurl.env` in any folder defines variables for its scenarios and every
  sub-folder; deeper folders override shallower ones. Secrets are redacted in logs and reports.
- **Visual builder** for everything HURL has: methods, query/form/**multipart** (files from the scenario's folder),
  cookies, basic auth, JSON/XML/text/GraphQL/base64/hex/file bodies, per-request options, asserts (every query,
  filter and predicate), captures, response status/version/headers and body assertions. Two-way with a text editor
  that has syntax highlighting and inline diagnostics.
- **Run** a request, a scenario, a folder or a whole workspace (parallel). Inspect bodies, headers, cookies,
  asserts, captures, a timing waterfall, cURL and TLS details.
- **Reports** in the app, run history, and export of HURL's own HTML / JUnit / TAP / JSON reports.
- **Looks like a Mac app everywhere**: GitHub light/dark theme, SF/Inter UI font, Menlo for code and URLs. Real
  native window chrome on macOS; a drawn macOS-style titlebar on Linux and Windows.

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
