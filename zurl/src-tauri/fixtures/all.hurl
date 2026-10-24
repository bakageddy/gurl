# every body/section kind
GET https://example.org/a
Accept: */*
[Query]
a: 1
b: {{b}}
[BasicAuth]
bob: secret
[Cookies]
session: abc
HTTP 200

POST https://example.org/form
[Form]
user: {{user}}
tags: a
HTTP *

POST https://example.org/upload
[Multipart]
field: hello
upload: file,data.bin; application/octet-stream
HTTP/1.1 201

POST https://example.org/raw
Content-Type: application/xml
<root><a>1</a></root>
HTTP 200

PUT https://example.org/b64
base64,SGVsbG8=;
HTTP 200

PUT https://example.org/hex
hex,48656c6c6f;
HTTP 200

PUT https://example.org/file
file,body.json;
HTTP 200

POST https://example.org/graphql
```graphql
query Q($id: ID!) { user(id: $id) { name } }
variables {"id": 1}
```
HTTP 200
[Asserts]
jsonpath "$.data.user.name" == "x"
jsonpath "$.list" count == 3
jsonpath "$.list" nth 0 isString
xpath "string(//a)" matches /a+/
header "Set-Cookie" contains "x"
cookie "s[Secure]" exists
certificate "Subject" contains "CN"
bytes count > 0
sha256 == hex,aa;
variable "v" == 1
ip isIpv4
body == ```
multi
```
url startsWith "https"
redirects count == 0
jsonpath "$.x" replaceRegex /a/ "b" not == "c"

GET https://example.org/captures
[Options]
location: true
retry: 3
retry-interval: 500ms
variable: host=example.org
header: X-Test: 1
HTTP 200
[Captures]
tok: jsonpath "$.t"
csrf: xpath "string(//m/@c)" redact
d: duration
n: header "X" regex "(\\d+)" toInt
{"ok": true, "n": [1, 2]}
