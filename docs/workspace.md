# Workspace

A workspace is a folder on disk. gURL shows its content in the sidebar. A workspace can contain:

- scenario files (`.hurl`),
- sub-folders, up to 255 levels deep,
- one `gurl.env` file per folder, with variables,
- one `gurl.headers` file per folder, with headers,
- data files that scenarios refer to.

Files that start with a dot are hidden. Other file types are not shown.

## Open a workspace

1. Click the folder icon in the sidebar header, or press Ctrl/Cmd+Shift+O.
2. Select a folder.

gURL remembers the last workspace and opens it at the next start.

To open one `.hurl` file that is outside the workspace, press Ctrl/Cmd+O. The file opens in a tab. Data file
references in that file are resolved from the folder of the file.

## The file tree

- Click a folder to expand or collapse it.
- Click a scenario to open it in a tab.
- Click a `gurl.env` or `gurl.headers` file to open the variables and headers tab of its folder.
- Right-click an item for its menu: new scenario, new folder, import, variables and headers, run, rename,
  duplicate, delete.
- The key icon on a folder row opens the variables and headers tab. The icon is highlighted when the folder has
  a `gurl.env` or a `gurl.headers`.

The sidebar has two views: **Files** and **History**. See [Reports and history](reports.md) for History.

## Create scenarios and folders

- Click the file icon in the sidebar header, or press Ctrl/Cmd+N, to create a scenario. Enter a name. The
  extension `.hurl` is added when it is missing.
- Click the folder icon with a plus sign to create a folder.
- Both actions use the selected folder as the target. When a file is selected, its folder is the target. When
  nothing is selected, the workspace root is the target.

Names cannot contain `/`, `\` or a null character. gURL refuses paths that leave the workspace.

## Variables: `gurl.env`

Each folder can hold one `gurl.env` file. The file defines variables for the scenarios in the folder and in
all its sub-folders.

Format:

```
# a comment
host=https://api.example.org
# @secret
token=abc123
```

- One `name=value` per line.
- Spaces around `=` are removed.
- A line that starts with `#` is a comment.
- The comment `# @secret` marks the next variable as a secret. gURL redacts secret values in logs and reports.

Inheritance:

- The files are read from the workspace root down to the folder of the scenario.
- A deeper folder overrides a variable of the same name from a parent folder.
- Variables from the run settings override all `gurl.env` files. See [Running](running.md).

Use a variable in a scenario with the HURL template syntax `{{name}}`.

### Edit variables in the application

1. Open the variables and headers tab of a folder: click the key icon on the folder row, or use **Variables &
   headers...** in the folder menu, or click the key icon in the sidebar header for the workspace root.
2. Use the **Table** view to add, edit and remove variables. Tick **secret** to mark a secret.
3. Use the **gurl.env** view to edit the raw text.

Changes are saved automatically. When the last variable is removed, the file is deleted. The **Inherited from
parent folders** list shows the variables that come from parent folders. Click **Override** to define the same
name in this folder.

## Common headers: `gurl.headers`

Each folder can hold one `gurl.headers` file. The file adds headers to every request of the scenarios in the
folder and in all its sub-folders.

Format:

```
# sent with every request below this folder
Authorization: Bearer {{token}}
Accept: application/json
```

- One `Name: value` per line.
- A line that starts with `#` is a comment.
- A line without `:` is ignored.
- A value can contain `{{name}}`. gURL replaces it with the value of the variable before the request is sent.
  Only plain variable names are supported. HURL functions and filters are not supported in this file.
- When a variable in a value is not defined, the run fails with an error. gURL never sends the literal
  `{{name}}` text.

Inheritance and precedence:

- The files are read from the workspace root down to the folder of the scenario.
- A deeper folder overrides a header of the same name from a parent folder. Names are compared without regard
  to case.
- A header from `gurl.headers` replaces a header of the same name in the scenario. This is the behaviour of the
  HURL option `--header`, which gURL uses to add the headers.
- On the command line, `--header` overrides `gurl.headers`. See [Command line](cli.md).

### Edit headers in the application

1. Open the variables and headers tab of a folder.
2. Scroll to the **Headers** section.
3. Edit the text. Changes are saved automatically. When the text is empty, the file is deleted.

The **Inherited from parent folders** list shows the headers that come from parent folders. Click **Override**
to add the same header to this folder.

## Data files

Multipart file parts and `file,...;` bodies refer to files on disk. HURL resolves these paths from the folder
of the scenario. gURL applies one rule: the file must be in the folder of the scenario or in one of its
sub-folders.

- In the builder, click **Choose...** next to a file field to select a file. gURL writes the relative path.
- A reference that points outside the folder is shown as an error in the builder.

## Limits

- A workspace can be 255 folder levels deep. gURL refuses to create a deeper folder.
- Response bodies larger than 2 MiB are cut in the result view. Click **Load full body** to load the rest.
