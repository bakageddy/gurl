# gURL user documentation

gURL is a desktop application for HURL. HURL is a text format and a command line tool for HTTP requests and
tests. gURL shows HURL files in a visual builder and in a text editor, runs them, and shows the results. The
same program is also the HURL command line.

This documentation is written in Simplified Technical English. Each page describes one area of the
application.

| Page | Content |
| --- | --- |
| [Installation](install.md) | Supported platforms, packages, runtime dependencies |
| [Workspace](workspace.md) | Folders, scenario files, `gurl.env` variables, `gurl.headers` headers, data files |
| [Scenarios](scenarios.md) | The visual builder, the text editor, formatting |
| [Running](running.md) | Run scopes, run settings, the result view, response tools, click-to-assert |
| [Quick request](quick-request.md) | The scratch tab with a URL bar |
| [Import](import.md) | cURL, Postman and OpenCollection import |
| [Reports and history](reports.md) | Report tabs, run history, export |
| [Command line](cli.md) | `gurl cli`, `gurl fmt`, `gurl import`, `gurl --version` |
| [Keyboard shortcuts](shortcuts.md) | All shortcuts |

## Terms

- **Scenario**: a `.hurl` file. A scenario contains one or more requests.
- **Request**: one entry of a scenario. A request has a method, a URL, optional sections and an optional
  response block.
- **Response block**: the part of a request that starts with `HTTP`. It contains the expected status, headers,
  asserts, captures and body.
- **Workspace**: the folder that you open in gURL. It contains scenarios, sub-folders and configuration files.
- **Run**: one execution of a request, a scenario, a folder or a workspace.

## HURL reference

gURL does not change the HURL format. For the format itself, see the HURL documentation at
https://hurl.dev/docs/hurl-file.html. The HURL version is shown by `gurl --version`.
