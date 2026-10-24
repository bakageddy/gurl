# Contributing to Zurl

Thanks for helping out! Zurl is a [Tauri 2](https://tauri.app) app: a Rust backend that embeds the
[`hurl`](https://crates.io/crates/hurl) crates (no sidecar, no external `hurl` binary) and a
Svelte 5 frontend. The app lives in the `zurl/` directory.

## Getting set up

You need Node.js 22+, a recent stable Rust toolchain and the Tauri system dependencies for your OS.

**Ubuntu / Debian**

```sh
sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

**Fedora**

```sh
sudo dnf install -y webkit2gtk4.1-devel openssl-devel gtk3-devel libappindicator-gtk3-devel \
  librsvg2-devel libxdo-devel gcc gcc-c++ make patchelf
```

**Arch**

```sh
sudo pacman -S --needed base-devel webkit2gtk-4.1 gtk3 libappindicator-gtk3 librsvg xdotool openssl patchelf
```

**macOS**: install the Xcode command line tools (`xcode-select --install`).
**Windows**: install the Visual Studio C++ build tools; WebView2 ships with Windows 11.

Then:

```sh
cd zurl
npm install
npm run tauri dev      # app with hot reload
```

## Project layout

```
zurl/
  src/                      Svelte 5 frontend (runes, plain Vite SPA)
    lib/api.ts              typed wrappers over every Tauri command
    lib/types.ts            mirrors the Rust model (camelCase JSON)
    lib/hurl-schema.ts      tables that drive the builder forms (queries, filters, predicates, options)
    lib/stores/*.svelte.ts  state: workspace, tabs (model <-> text sync), runs, settings, ui
    components/             Titlebar/chrome, Sidebar, DocView, builder/*, run/*, VarsPanel
  src-tauri/src/
    parse.rs / emit.rs      HURL text <-> structured model (model.rs) for the visual builder
    workspace.rs            workspace tree, zurl.env variable inheritance, file ops, 255-level limit
    runs.rs                 run scopes (text/file/dir), streaming, history, HURL report export
    hurl.rs                 runner options + one run, built on the hurl crate
    lib.rs                  Tauri commands, app state, file watcher
  src-tauri/fixtures/       HURL files used by the round-trip tests
packaging/                  desktop entry and Arch PKGBUILD
```

### Concepts worth knowing

- **Workspace**: a folder tree up to 255 levels deep. Each folder can hold many `.hurl` scenarios and one
  `zurl.env` (`name=value` lines, `# @secret` above a line marks a secret). A folder's variables apply to its
  scenarios and all sub-folders; deeper folders override shallower ones.
- **Builder <-> text**: both views edit one model. `parse_hurl` turns text into the model, `emit_hurl` turns it
  back. Leaf values stay raw HURL source, so `{{templates}}` and quoting survive. If you add support for new
  syntax, extend `model.rs`, `parse.rs`, `emit.rs` **and** add it to `src-tauri/fixtures/all.hurl`; the
  round-trip test fails if anything is lost.
- **Data files** (multipart parts, `file,...;` bodies) must live in the scenario's folder or a sub-folder
  (HURL's file root). The builder's picker and the runner both enforce this.

## Before you open a pull request

```sh
cd zurl
npm run check                       # svelte-check / TypeScript
npm run build                       # production frontend build

cd src-tauri
cargo test --lib                    # unit + round-trip + run/export tests
cargo clippy --lib --tests -- -D warnings
cargo fmt
```

CI runs the same checks on Ubuntu 22.04/24.04, macOS and Windows (`.github/workflows/ci.yml`).

## Style

- Match the surrounding code: comment density, naming and idiom.
- UI: use the CSS tokens in `src/app.css` (GitHub Primer colours; `--font-ui`, `--font-mono`). Don't hard-code
  colours or fonts. Code and URLs use the mono stack (Menlo on macOS, bundled DejaVu Sans Mono elsewhere).
- Keep the frontend fast: no new heavy dependencies on the startup path; lazy-load anything big (CodeMirror is
  loaded on first use via `LazyEditor.svelte`); virtualise long lists.
- Keep platform differences behind `src/lib/platform.ts` (native macOS chrome vs the drawn macOS-style chrome).
- Commit messages: short imperative subject, details in the body if needed.

## Releases

Releases are cut by tag. Bump the version in `zurl/package.json`, `zurl/src-tauri/Cargo.toml` and
`zurl/src-tauri/tauri.conf.json`, then:

```sh
git tag v0.2.0 && git push origin v0.2.0
```

`.github/workflows/release.yml` builds and attaches to the GitHub release:

| Platform | Artifact |
| --- | --- |
| macOS (Apple Silicon and Intel) | `.dmg` |
| Windows | NSIS installer `.exe` |
| Ubuntu 22.04 and 24.04 | `.deb` and `.AppImage` (suffixed per baseline) |
| Fedora 42 | `.rpm` (built in a Fedora container) |
| Arch | `.tar.gz` binary; `packaging/arch/PKGBUILD` builds a pacman package |

Bundles are unsigned (right-click -> Open on macOS, SmartScreen notice on Windows).

## Reporting bugs

Please include your OS, the Zurl version, and, if a scenario misbehaves, the smallest `.hurl` file that
reproduces it.
