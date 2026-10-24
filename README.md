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
- **Response tools**: pretty / raw / tree / preview views, search, a JSONPath / XPath box evaluated by HURL's own
  engine, and **click-to-assert**: any JSON value, header, cookie, status, duration or TLS field becomes an assert
  (or a capture) in the scenario, inserted without touching the rest of the file.
- **Reports** in the app, run history, and export of HURL's own HTML / JUnit / TAP / JSON reports.

## Install

Download the latest build for your platform from the Releases page: `.dmg` (macOS, Apple Silicon and Intel),
NSIS `.exe` (Windows), `.deb`/`.AppImage` (Ubuntu 22.04 and 24.04), `.rpm` (Fedora), or the Arch tarball /
`packaging/arch/PKGBUILD`.

### Runtime dependencies

gURL links the HURL engine's native libraries dynamically from the OS: **libcurl**, **libxml2** and **OpenSSL**.
The `.deb`, `.rpm` and the Arch `PKGBUILD` declare them, so a package manager pulls them in. For the AppImage or
the Arch tarball, make sure they are installed:

| | Packages |
| --- | --- |
| Ubuntu / Debian | `libcurl4 libxml2 libssl3` (`libssl3t64` on 24.04) |
| Fedora | `libcurl libxml2 openssl-libs` |
| Arch | `curl libxml2 openssl` |
| macOS | nothing: libcurl and libxml2 ship with the OS |
| Windows | nothing: the installer is self-contained |

`gurl --version` prints which libraries are in use.

## Command line

The same binary is HURL's command line, so one install covers the desktop and CI:

```sh
gurl                                   # open the app
gurl cli --test api/*.hurl             # exactly `hurl --test api/*.hurl`
gurl cli --report-html out login.hurl  # every hurl option, HURL_* env var and exit code (0/1/2/3/4)
gurl --version                         # gURL, HURL, libcurl/libxml2 versions and how they are linked
gurl fmt --check .                     # `hurlfmt` (format / --in-place / --out json|html / --in curl); directories expand to every .hurl
```

`gurl cli` adds the workspace to hurl: for each file it finds the topmost parent directory holding a `gurl.env`
and applies the variables from there down to the file's directory (deeper overrides shallower, `# @secret`
values are redacted), exactly as the app does. `--workspace <DIR>` sets the root explicitly, `--no-workspace`
turns it off, and `--variable`/`--variables-file`/`HURL_VARIABLE_*` still win over `gurl.env`.

Putting `gurl` on your PATH: Linux packages install `/usr/bin/gurl`. On macOS, `ln -s /Applications/gURL.app/Contents/MacOS/gurl /usr/local/bin/gurl`.
On Windows, add the install directory to PATH; the terminal prompt may return before the output when the
command is not piped or redirected (the app is a GUI program), `gurl cli … | more` waits.

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
  for code; both are bundled. [HURL](https://hurl.dev) by Orange does the actual requests; `gurl cli` is HURL's own
  command line source (Apache-2.0), vendored in `gurl/crates/gurl-core/src/cli/hurl`.
