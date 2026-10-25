# Import

gURL converts three formats into `.hurl` scenarios: cURL commands, Postman collections and OpenCollection
YAML. The result is a set of folders and scenario files, plus `gurl.env` files for variables. The same
conversion is available on the command line as `gurl import`. See [Command line](cli.md).

## Open the import dialog

- Click the download icon in the sidebar header. The target is the selected folder.
- Or use **Import here...** in the menu of a folder.

The dialog has three tabs: **cURL**, **Postman**, **OpenCollection**. The right side shows a preview of the
files that will be created, and the warnings. Click **Import** to write the files. gURL does not overwrite
existing files. When a target file exists, the import stops with an error and writes nothing.

Option **One scenario per folder**: by default, each request becomes one `.hurl` file. With this option, the
requests of a folder are written into one `.hurl` file, in order. Captures of one request are then available
to the next ones.

## cURL

Paste one or more cURL commands. Commands can be separated by new lines, `;` or `&&`. Quotes, backslash
escapes, line continuations and `$'...'` strings are understood. Commands copied from the browser developer
tools ("Copy as cURL", bash and cmd variants) are supported.

Supported options:

| Option | Result |
| --- | --- |
| `-X`, `--request` | Method |
| `--url`, positional URL | URL |
| `-H`, `--header` | Header. `Cookie` becomes a `[Cookies]` section. |
| `-d`, `--data`, `--data-raw`, `--data-binary`, `--data-urlencode` | Body. Form pairs become `[Form]`. JSON becomes a JSON body. `@file` becomes a `file,...;` body. |
| `--json` | JSON body with the content type header |
| `-F`, `--form` | `[Multipart]` section. `@file` becomes a file part with an optional type. |
| `-G`, `--get` | Data becomes `[Query]` parameters |
| `-u`, `--user` | `[BasicAuth]` section |
| `-A`, `-e`, `-b` | User agent, referer and cookie headers |
| `-k`, `-L`, `--max-redirs`, `-m`, `--connect-timeout`, `-x`, `--retry`, `--compressed`, `--http1.1`, `--http2`, `-I`, `--digest`, `--ntlm`, `--negotiate`, `--resolve`, `--cacert`, `-E`, `--key` | `[Options]` entries |

Options that are not in this list are reported as warnings. The command is converted without them. Each
command gets a response block `HTTP *`.

## Postman

Select a collection file in Postman Collection Format v2.1. In Postman, use **File**, **Export**.

Conversion:

- Folders become folders. Requests become scenarios. Names are converted to file names in lower case with
  hyphens. Duplicate names get a number.
- Request: method, URL, headers (disabled headers are skipped), query parameters, path variables `:id` (written
  as `{{id}}`), and the body: raw text or JSON, URL-encoded form, form data with text and file parts, a file
  body, or a GraphQL body.
- Authentication: basic becomes `[BasicAuth]`. Bearer becomes an `Authorization` header. API key becomes a
  header or a query parameter. Other types are reported as warnings. Authentication set on the collection or a
  folder applies to the requests below it.
- Collection variables become `gurl.env` in the target folder.
- Test scripts, pre-request scripts and saved examples have no HURL equivalent. They are reported as warnings.

## OpenCollection

Select one of:

- a bundled YAML file,
- the folder layout that Bruno writes: `opencollection.yml`, `folder.yml` files, one YAML file per request, and
  an `environments` folder.

Conversion:

- Folders become folders. HTTP requests become scenarios. GraphQL requests become scenarios with a GraphQL body.
- Request: method, URL, headers, query and path parameters, body (JSON, text, XML, SPARQL, URL-encoded form,
  multipart form, file).
- Authentication: basic, bearer, API key, digest, NTLM, AWS SigV4. `inherit` uses the folder or collection
  default.
- Assertions on the status, the body and the headers become HURL asserts when the operator has a HURL
  equivalent. Other assertions are written as comments.
- Variables of the collection, the folders and the environments become `gurl.env` files. Secret variables are
  marked with `# @secret` and written with an empty value.
- Settings for timeout and redirects become `[Options]` entries.
- gRPC, WebSocket and script items are reported as warnings.
