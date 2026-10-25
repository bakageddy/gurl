# Scenarios

A scenario is a `.hurl` file. gURL shows a scenario in two views: the **Builder** and the **Text** view. Both
views edit the same file. Use the switch in the toolbar of the tab to change the view.

## Builder view

The builder shows one card per request. The cards are in the order of the file.

### Add, move and remove requests

- Click **Add request at top** to insert a request before the first one.
- Click **Add request** at the bottom to append a request.
- Use the arrow buttons on a card to move the request up or down.
- Click the copy icon to duplicate a request.
- Click the trash icon to delete a request.

### Request

Each card has these fields:

- **Comment**: text that is written as `#` lines above the request.
- **Method** and **URL**. The URL can contain `{{variables}}`.
- **Headers**: name and value pairs.
- **Query** parameters, written as a `[Query]` section.
- **Form** fields, written as a `[Form]` section.
- **Multipart** parts, written as a `[Multipart]` section. A part is a text value or a file. For a file, select
  it with **Choose...**. The file must be in the folder of the scenario or below it. An optional content type
  can be set.
- **Cookies**, written as a `[Cookies]` section.
- **Basic auth**: a user name and a password, written as a `[BasicAuth]` section.
- **Options**: per-request HURL options, written as an `[Options]` section. Examples: `location`, `retry`,
  `retry-interval`, `variable`, `header`.
- **Body**: one of JSON, XML, text, GraphQL, a multiline string, base64, hex, or a file reference.

### Response block

Click **Add response checks (status, asserts, captures)** to add a response block. The block has these fields:

- **Status**: a status code, or `*` for any status.
- **HTTP version**: optional, for example `HTTP/1.1`.
- **Headers**: expected response headers.
- **Asserts**: a list of asserts. Each assert has a query, optional filters, a predicate and a value. Tick
  **not** to negate the predicate. All HURL queries, filters and predicates are available.
- **Captures**: a list of captures. Each capture has a variable name, a query and optional filters. Tick
  **redact** to hide the value in logs and reports.
- **Body**: the expected body.

Note: HURL compares the expected body with the response body byte for byte. White space and the final newline
are part of the comparison. For a structural check of JSON, use `jsonpath` asserts. See
[Running](running.md) for the **Expect this body** action.

Click the cross icon next to the response block to remove it.

## Text view

The text view is a code editor with:

- syntax highlighting for HURL,
- error markers from the HURL parser, shown inline,
- code folding for sections and bodies,
- search and replace (Ctrl/Cmd+F inside the editor),
- undo and redo.

Changes in the text view are parsed and shown in the builder. Changes in the builder are written back as
text.

## Comments and formatting

The builder keeps comments that are written above a request. Comments in other positions are kept as long as
you edit in the text view. When you edit in the builder, the file is rewritten from the model, and comments in
other positions are removed. The builder shows a warning at the top when the file contains such comments, and
when it contains syntax that the builder cannot show.

Asserts and captures that you add from a result view are inserted into the text directly. They do not cause a
rewrite. See [Running](running.md).

## Format

Click **Format**, or press Shift+Alt+F, to format the scenario with `hurlfmt`. The formatter is part of HURL.
When the file does not parse, gURL shows the parser error and does not change the text.

## Save

Press Ctrl/Cmd+S to save. A tab with unsaved changes shows a dot in its title. For a new scenario without a
file, a file dialog asks for the location.

## Supported syntax

gURL supports the full HURL syntax of the bundled HURL version in the text view. The builder supports all
request sections, all body kinds, all asserts, filters and captures, and the response status, version, headers
and body. A file that uses syntax the builder cannot show is still editable in the text view.
