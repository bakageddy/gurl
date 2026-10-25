# Command line

The application binary is also a command line tool. See [Installation](install.md) for how to put it on your
`PATH`.

```
gurl                        open the application
gurl cli [OPTIONS] [FILE]...  run HURL files, exactly like `hurl`
gurl fmt [OPTIONS] [FILE]...  format, check or export HURL files, like `hurlfmt`
gurl import <FORMAT> <FILE|->  convert cURL, Postman or OpenCollection to HURL
gurl --version              versions of gURL, HURL and the native libraries
gurl --help                 this list
```

## `gurl cli`

`gurl cli` is the HURL command line. It accepts every `hurl` option and every `HURL_*` environment variable,
produces the same output and the same reports, and returns the same exit codes.

| Exit code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | Failed to initialize |
| 2 | A file could not be parsed |
| 3 | A runtime error occurred |
| 4 | An assert failed |

Example:

```sh
gurl cli --test api/*.hurl
gurl cli --report-html out login.hurl
gurl cli --variable host=https://api.example.org --json a.hurl
```

For the options, run `gurl cli --help`, or see https://hurl.dev/docs/manual.html.

### Workspace mode

`gurl cli` adds the gURL workspace to HURL. For each input file:

1. It finds the workspace root: the topmost parent folder that contains a `gurl.env` or a `gurl.headers`.
2. It reads the `gurl.env` files from the root down to the folder of the file. A deeper file overrides a
   parent. Variables marked `# @secret` are redacted in the output and the reports.
3. It reads the `gurl.headers` files in the same way, renders `{{name}}` in the values with the variables, and
   adds the headers to every request of the file.

Precedence of variables, from lowest to highest: `gurl.env` chain, HURL configuration file, `HURL_*`
environment variables, `--variables-file`, `--variable` and `--secret`.

Precedence of headers, from lowest to highest: headers in the scenario, `gurl.headers` chain, `--header`.

Options:

| Option | Effect |
| --- | --- |
| `--workspace <DIR>` | Use `DIR` as the workspace root instead of the discovered root. |
| `--no-workspace` | Do not read any `gurl.env` or `gurl.headers`. |

Input from standard input has no folder. It uses the current directory, and only with `--workspace`.

## `gurl fmt`

`gurl fmt` is `hurlfmt`, with two additions: a directory argument expands to every `.hurl` file below it, and
`--workspace <DIR>` does the same for `DIR`.

| Option | Effect |
| --- | --- |
| `--check` | Report files that are not formatted. Exit code 3 when a file needs formatting. |
| `--in-place` | Write the formatted text back to the files. |
| `--in hurl\|curl` | Input format. `curl` converts a cURL command with the `hurlfmt` converter. |
| `--out hurl\|json\|html` | Output format. |
| `-o`, `--output <FILE>` | Write to `FILE` instead of standard output. |
| `--standalone` | Produce a standalone HTML page with `--out html`. |
| `--color`, `--no-color` | Colored output on or off. |
| `--workspace <DIR>` | Format or check every `.hurl` file below `DIR`. |

Exit codes: 0 success, 1 error, 2 invalid input, 3 a file is not formatted (with `--check`).

Examples:

```sh
gurl fmt --check .
gurl fmt --in-place api/
gurl fmt --out json login.hurl
```

Note: the `--in curl` converter is the one from `hurlfmt`. It supports fewer options than `gurl import curl`.

## `gurl import`

```
gurl import <curl|postman|opencollection> <FILE|-> [--out DIR] [--one-file]
```

| Argument or option | Meaning |
| --- | --- |
| `curl`, `postman`, `opencollection` | Input format. |
| `FILE` | A file, a directory for the OpenCollection folder layout, or `-` for standard input. |
| `--out <DIR>` | Write the `.hurl` files and `gurl.env` files below `DIR`. Without it, the files are printed. |
| `--one-file` | One `.hurl` file per folder, with the requests in order, instead of one file per request. |

Exit code 2 when the input cannot be parsed. Existing files are not overwritten. See [Import](import.md) for
what each format converts to.

Example:

```sh
pbpaste | gurl import curl -
gurl import postman collection.json --out api/
```

## `gurl --version`

Prints:

- the gURL version and the target triple,
- the HURL version,
- the libcurl version line as HURL prints it, with the TLS library, zlib, brotli and nghttp2 versions,
- the libcurl features,
- for libcurl and for libxml2: the version and whether the library is linked statically or dynamically. For a
  dynamic library, the path of the loaded file is shown.

`gurl cli --version` prints the same text.
