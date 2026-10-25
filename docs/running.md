# Running

## Run scopes

You can run:

- **The active scenario**: click **Run** in the tab toolbar, or press Ctrl/Cmd+Enter. gURL runs the text in
  the editor, saved or not.
- **One scenario from the sidebar**: use **Run** in the file menu.
- **A folder**: use **Run this folder** or **Run folder recursively** in the folder menu.
- **The workspace**: use **Run workspace** in the sidebar header menu.

A folder run opens a report tab. The files run in parallel. The number of parallel files is a run setting.
Click **Cancel** in the report tab to stop a run. Files that did not start are skipped.

## Variables and headers in a run

For each file, gURL reads the `gurl.env` chain and the `gurl.headers` chain from the workspace root down to
the folder of the file. See [Workspace](workspace.md). The variable overrides from the run settings win over
`gurl.env`.

A quick request has no file. It uses the chain of the folder that is selected in the sidebar.

## Run settings

Click the gear icon at the bottom of the sidebar to open the run settings. The settings apply to all runs.

| Group | Setting |
| --- | --- |
| Workspace runs | Include sub-folders when running a folder. Number of parallel files (1 to 16). |
| Variable overrides | One `name=value` per line. These override every `gurl.env`. |
| Connection | Follow redirects. Forward credentials to other hosts. Maximum redirects. Skip TLS certificate verification. Request compressed responses. Keep cookies between requests. HTTP version. IP version. |
| Timing | Timeout. Connect timeout. Retries. Retry interval. Delay between requests. Repeat count. |
| Identity and network | User agent. User and password. Proxy. No-proxy list. CA certificate. Client certificate. Client key. |
| Behaviour | Stop a scenario at the first failing request. Ignore asserts. |

The settings map to the HURL options with the same names.

## The result view

After a run of one scenario, the result view opens below the scenario. For a folder run, select a file in the
report tab to see its result view.

The result view has:

- a list of the requests of the scenario, with status and duration. Click a request to select it.
- the status line of the selected request: status code, HTTP version, duration and body size. Click the
  status or the duration to add an assert on it.
- tabs: **Body**, **Headers**, **Cookies**, **Asserts**, **Captures**, **Timings**, **cURL**, **TLS**.

When a request is retried or follows redirects, the last call is shown.

### Body tab

The body tab has these views. The available views depend on the content type.

| View | Content |
| --- | --- |
| Pretty | JSON with indentation. |
| Raw | The body text as received. |
| Tree | A tree of a JSON body. Expand and collapse nodes. Use the filter field to find keys and values. |
| Preview | An HTML page in a sandbox without scripts or network access, or an image. |

Other controls:

- **Search**: opens the search panel of the editor. Ctrl/Cmd+F inside the body does the same.
- **Query**: opens the query bar. See below.
- **Expect this body**: writes the response body into the scenario as the expected body, byte for byte. Use
  this when the scenario must check the whole body. gURL selects a form that HURL evaluates to exactly these
  bytes: a bare JSON value when HURL renders it unchanged, otherwise a fenced multiline string. When the body
  has no final newline, the closing fence is written on the last line of the text.
- **Copy**: copies the body.
- **Load full body**: shown when the body is larger than 2 MiB. The first 2 MiB are shown until you click it.

The view that you select is remembered for the same kind of content during the session.

### Query bar

The query bar evaluates a JSONPath expression on a JSON body, or an XPath expression on an XML or HTML body.
HURL's own engine does the evaluation. The result is the value that a HURL assert would see.

1. Click **Query**.
2. Type an expression, for example `$.items[0].id` or `count(//item)`.
3. The result is shown below the field: a value, a list, or `none`, and an error for an invalid expression.
4. Select a predicate from the list. The assert line that would be added is shown.
5. Click **Add assert** to add the assert to the scenario, or **Capture...** to add a capture.

### Headers, Cookies, Timings, cURL and TLS tabs

- **Headers** and **Cookies** list the response headers and the cookies that were set. Click a row to add an
  assert on it.
- **Timings** shows a waterfall of the libcurl timings: DNS lookup, TCP connect, TLS handshake, request sent,
  waiting for the first byte, download.
- **cURL** shows the request as a cURL command. Click **Copy** to copy it.
- **TLS** shows the certificate fields of an HTTPS response. Click a field to add an assert on it.

### Asserts tab

The asserts tab lists every assert of the request, implicit and explicit, with its source line and result.
`HTTP 200` is one row. It fails when the version or the status does not match.

For a failed assert, the tab shows the details that HURL recorded:

- **Expected** and **actual** values for status, version, header and explicit asserts.
- **Why** the assert could not be evaluated, for example "header not found" or "the body is not valid JSON".
- For a body assert, a **diff** from the expected body to the actual body. Removed lines are the expected
  text. Added lines are the actual text. Long runs of equal lines are folded. Click a fold to expand it. A
  line without a final newline is marked. When the two bodies are equal except for white space, the final
  newline or the key order, the tab says so in one line.

Note: when the status assert fails, HURL does not evaluate the other asserts of that request. The tab then
shows only the status row.

### Captures tab

The captures tab lists the variables that the request captured, with their values.

## Click-to-assert

Many values in the result view can be turned into an assert in the scenario:

- a JSON node in the tree view (hover the node and click the plus sign, or click the node),
- a response header,
- a cookie,
- the status code,
- the duration,
- a TLS certificate field,
- the result of the query bar.

A menu opens with suggested asserts. The first suggestion is the most specific one, for example
`jsonpath "$.id" == 42`. Other suggestions use other predicates, for example `exists`, `startsWith`,
`isInteger`, `count ==`, `isIsoDate` or `isUuid`, depending on the value. The menu also offers:

- **Capture as variable...**: adds a capture with the same query. Enter the variable name.
- **Copy** actions: the JSONPath, the value, or the assert line.

When you select an assert, gURL inserts the line into the scenario text:

- into the `[Asserts]` or `[Captures]` section of the request when it exists,
- otherwise into a new section after the existing sections and before the body,
- when the request has no response block, after a new `HTTP *` line.

Only the new lines are inserted. The rest of the file, including comments, is not changed. The text view
scrolls to the new line. When the same line is already present, gURL reports it and does not add a duplicate.

Click-to-assert is available for the result of the active scenario, and for files in a report tab. For a report
tab, gURL opens the file and inserts the line. It is not available while a run is in progress, or for a quick
request that is not saved.

When the scenario text changed since the run, gURL still inserts the line at the request with the same
position, and shows a warning.
