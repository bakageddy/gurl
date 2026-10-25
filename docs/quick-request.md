# Quick request

A quick request is a scratch scenario with one request and a URL bar. Use it to try a request before you add
it to the workspace.

## Open a quick request

Press Ctrl/Cmd+K. A new tab opens. When a quick request tab is already open and not changed, that tab is
activated instead.

## Use the URL bar

1. Select the method.
2. Type the URL. The URL can contain `{{variables}}`.
3. Press Enter, or click **Send**.

The request runs with the run settings and with the `gurl.env` and `gurl.headers` chain of the folder that
is selected in the sidebar. See [Workspace](workspace.md).

The builder below the URL bar shows the full request. Add headers, a body, asserts and captures there. The
result view opens below after the request is sent.

## Paste a cURL command

Paste a cURL command into the URL field. gURL converts the command and replaces the request: method, URL,
headers, cookies, body, basic auth and options. Options that have no HURL equivalent are reported as warnings.
See [Import](import.md) for the supported options.

## Save

Click **Save as...**, or press Ctrl/Cmd+S, to save the quick request as a scenario. A file dialog opens in the
workspace folder. Select the location and the name. The tab becomes a normal scenario tab.

## Close

Close the tab with Ctrl/Cmd+W or the close button. gURL does not ask for confirmation when the request was not
sent and not edited.
