import { StreamLanguage, HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";
import { EditorView } from "@codemirror/view";

const METHODS = /^(GET|POST|PUT|PATCH|DELETE|HEAD|OPTIONS|CONNECT|TRACE)\b/;
const SECTIONS = /^\[(Query|QueryStringParams|Form|FormParams|Multipart|MultipartFormData|Cookies|BasicAuth|Options|Captures|Asserts)\]/;
const QUERIES = /^(status|version|url|header|cookie|body|xpath|jsonpath|regex|variable|duration|bytes|rawbytes|sha256|md5|certificate|ip|redirects)\b/;
const PREDICATES = /^(==|!=|>=|<=|>|<|not|startsWith|endsWith|contains|includes|matches|exists|isBoolean|isCollection|isDate|isEmpty|isFloat|isInteger|isIpv4|isIpv6|isIsoDate|isList|isNumber|isObject|isString|isUuid)(?=\s|$)/;
const FILTERS = /^(base64Decode|base64Encode|base64UrlSafeDecode|base64UrlSafeEncode|charsetDecode|count|daysAfterNow|daysBeforeNow|decode|first|format|dateFormat|htmlEscape|htmlUnescape|last|location|nth|replace|replaceRegex|split|toDate|toFloat|toHex|toInt|toString|urlDecode|urlEncode|urlQueryParam|utf8Decode|utf8Encode)\b/;

interface State {
  fence: boolean;
  lineStart: boolean;
}

/** Lightweight tokenizer for HURL syntax (the Rust parser stays the source of truth). */
export const hurlLanguage = StreamLanguage.define<State>({
  startState: () => ({ fence: false, lineStart: true }),
  token(stream, state) {
    if (stream.sol()) state.lineStart = true;
    if (state.fence) {
      if (stream.sol() && stream.match(/^```/)) {
        state.fence = false;
        return "keyword";
      }
      stream.skipToEnd();
      return "string";
    }
    if (stream.eatSpace()) return null;
    if (stream.match(/^#.*/)) return "comment";
    if (state.lineStart && stream.match(/^```\w*/)) {
      state.fence = true;
      return "keyword";
    }
    if (stream.match(/^\{\{/)) {
      stream.skipTo("}}");
      stream.match("}}");
      return "variableName";
    }
    if (state.lineStart) {
      state.lineStart = false;
      if (stream.match(SECTIONS)) return "heading";
      if (stream.match(METHODS)) return "keyword";
      if (stream.match(/^HTTP(\/[\d.]+)?\b/)) return "keyword";
    }
    if (stream.match(/^"(?:[^"\\]|\\.)*"/)) return "string";
    if (stream.match(/^\/(?:[^\/\\\n]|\\.)+\//)) return "regexp";
    if (stream.match(/^-?\d+(\.\d+)?(ms|s|m)?\b/)) return "number";
    if (stream.match(/^(true|false|null)\b/)) return "atom";
    if (stream.match(PREDICATES)) return "operator";
    if (stream.match(QUERIES)) return "typeName";
    if (stream.match(FILTERS)) return "function";
    if (stream.match(/^(https?|file):\/\/\S+/)) return "link";
    if (stream.match(/^[A-Za-z0-9_.-]+(?=:)/)) return "propertyName";
    stream.next();
    return null;
  },
  languageData: { commentTokens: { line: "#" } },
});

const highlight = HighlightStyle.define([
  { tag: t.keyword, color: "var(--syn-keyword)", fontWeight: "600" },
  { tag: t.heading, color: "var(--syn-fn)", fontWeight: "600" },
  { tag: t.string, color: "var(--syn-string)" },
  { tag: t.regexp, color: "var(--syn-string)" },
  { tag: t.number, color: "var(--syn-number)" },
  { tag: t.atom, color: "var(--syn-number)" },
  { tag: t.comment, color: "var(--syn-comment)", fontStyle: "italic" },
  { tag: t.operator, color: "var(--syn-keyword)" },
  { tag: t.typeName, color: "var(--syn-number)" },
  { tag: t.function(t.variableName), color: "var(--syn-fn)" },
  { tag: t.variableName, color: "var(--syn-var)" },
  { tag: t.propertyName, color: "var(--syn-prop)" },
  { tag: t.link, color: "var(--accent)" },
  { tag: t.bool, color: "var(--syn-number)" },
]);

export const githubHighlight = syntaxHighlighting(highlight);

export const githubTheme = EditorView.theme({
  "&": {
    color: "var(--fg)",
    backgroundColor: "var(--canvas)",
    fontSize: "12.5px",
    height: "100%",
  },
  ".cm-scroller": { fontFamily: "var(--font-mono)", lineHeight: "1.6" },
  ".cm-content": { caretColor: "var(--fg)", padding: "8px 0" },
  ".cm-cursor, .cm-dropCursor": { borderLeftColor: "var(--fg)" },
  "&.cm-focused": { outline: "none" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection": {
    backgroundColor: "var(--selection)",
  },
  ".cm-gutters": {
    backgroundColor: "var(--canvas)",
    color: "var(--fg-subtle)",
    border: "none",
    paddingLeft: "4px",
  },
  ".cm-activeLine": { backgroundColor: "color-mix(in srgb, var(--row-hover) 60%, transparent)" },
  ".cm-activeLineGutter": { backgroundColor: "transparent", color: "var(--fg)" },
  ".cm-matchingBracket": { backgroundColor: "var(--accent-subtle)", outline: "none" },
  ".cm-lintRange-error": { backgroundImage: "none", textDecoration: "underline wavy var(--danger)" },
  ".cm-tooltip": {
    backgroundColor: "var(--canvas-overlay)",
    border: "1px solid var(--border)",
    borderRadius: "6px",
    boxShadow: "var(--shadow)",
    color: "var(--fg)",
  },
  ".cm-panels": { backgroundColor: "var(--canvas-subtle)", color: "var(--fg)", borderColor: "var(--border)" },
  ".cm-searchMatch": { backgroundColor: "var(--attention-subtle)", outline: "1px solid var(--attention)" },
});
