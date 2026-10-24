// Turns things the user clicks in a response (a JSON value, a header, a cookie, a duration…) into
// HURL asserts and captures. Pure functions, unit-tested; the UI only decides where to show them.

import type { JsonNode } from "./json-tree";
import type { AssertM, CaptureM, FilterM, QueryM } from "./types";

export interface Suggestion {
  label: string;
  assert: AssertM;
}

/** RFC 9535 JSONPath for `segments`: dot notation for identifier-like keys, brackets otherwise. */
export function jsonPathFor(segments: (string | number)[]): string {
  let out = "$";
  for (const seg of segments) {
    if (typeof seg === "number") out += `[${seg}]`;
    else if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(seg)) out += `.${seg}`;
    else out += `['${seg.replace(/\\/g, "\\\\").replace(/'/g, "\\'")}']`;
  }
  return out;
}

/** A HURL double-quoted string literal. */
export function hurlString(s: string): string {
  let out = '"';
  for (const ch of s) {
    switch (ch) {
      case "\\":
        out += "\\\\";
        break;
      case '"':
        out += '\\"';
        break;
      case "\n":
        out += "\\n";
        break;
      case "\r":
        out += "\\r";
        break;
      case "\t":
        out += "\\t";
        break;
      default: {
        const code = ch.codePointAt(0)!;
        out += code < 0x20 ? `\\u{${code.toString(16)}}` : ch;
      }
    }
  }
  return out + '"';
}

/** A HURL regex literal `/…/` for a literal match of `s`. */
export function regexLiteral(s: string): string {
  return `/${s.replace(/[.*+?^${}()|[\]\\/]/g, "\\$&")}/`;
}

const ISO_DATE = /^\d{4}-\d{2}-\d{2}(?:[T ]\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?(?:Z|[+-]\d{2}:?\d{2})?)?$/;
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const IPV4 = /^(?:25[0-5]|2[0-4]\d|1?\d?\d)(?:\.(?:25[0-5]|2[0-4]\d|1?\d?\d)){3}$/;
const IPV6 = /^[0-9a-f:]+:[0-9a-f:]*$/i;

function mk(query: QueryM, func: string, value = "", filters: FilterM[] = [], not = false): AssertM {
  return { query, filters, not, func, value };
}

/** Suggestions for one JSON node, most useful first. */
export function suggestionsForJson(node: JsonNode): Suggestion[] {
  const q: QueryM = { kind: "jsonpath", args: hurlString(jsonPathFor(node.path)) };
  const out: Suggestion[] = [];
  const push = (label: string, func: string, value = "", filters: FilterM[] = [], not = false) =>
    out.push({ label, assert: mk(q, func, value, filters, not) });

  switch (node.kind) {
    case "string": {
      const v = node.raw;
      push(`== ${hurlString(v.length > 40 ? v.slice(0, 37) + "…" : v)}`, "==", hurlString(v));
      if (ISO_DATE.test(v)) push("isIsoDate", "isIsoDate");
      if (UUID.test(v)) push("isUuid", "isUuid");
      if (IPV4.test(v)) push("isIpv4", "isIpv4");
      else if (v.includes(":") && IPV6.test(v)) push("isIpv6", "isIpv6");
      if (v.length > 0) {
        push(`startsWith ${hurlString(v.slice(0, Math.min(v.length, 12)))}`, "startsWith", hurlString(v.slice(0, Math.min(v.length, 12))));
        push("contains …", "contains", hurlString(v));
        push(`matches ${regexLiteral(v.slice(0, 24))}`, "matches", regexLiteral(v));
      }
      push("isString", "isString");
      push("exists", "exists");
      push("not exists", "exists", "", [], true);
      break;
    }
    case "number": {
      const isInt = /^-?\d+$/.test(node.raw);
      push(`== ${node.raw}`, "==", node.raw);
      push(`> ${node.raw}`, ">", node.raw);
      push(`>= ${node.raw}`, ">=", node.raw);
      push(`< ${node.raw}`, "<", node.raw);
      push(`<= ${node.raw}`, "<=", node.raw);
      push(isInt ? "isInteger" : "isFloat", isInt ? "isInteger" : "isFloat");
      push("isNumber", "isNumber");
      push("exists", "exists");
      break;
    }
    case "boolean":
      push(`== ${node.raw}`, "==", node.raw);
      push("isBoolean", "isBoolean");
      push("exists", "exists");
      break;
    case "null":
      push("== null", "==", "null");
      push("exists", "exists");
      push("not exists", "exists", "", [], true);
      break;
    case "array": {
      const count: FilterM[] = [{ name: "count", args: "" }];
      push(`count == ${node.count}`, "==", String(node.count), count);
      push("isCollection", "isCollection");
      push(node.count === 0 ? "isEmpty" : "not isEmpty", "isEmpty", "", [], node.count !== 0);
      const first = node.children?.find((c) => c.kind === "string" || c.kind === "number" || c.kind === "boolean");
      if (first) push(`includes ${scalarLiteral(first)}`, "includes", scalarLiteral(first));
      push("exists", "exists");
      break;
    }
    case "object": {
      push("exists", "exists");
      push("isCollection", "isCollection");
      push(`count == ${node.count}`, "==", String(node.count), [{ name: "count", args: "" }]);
      break;
    }
  }
  return out;
}

function scalarLiteral(node: JsonNode): string {
  return node.kind === "string" ? hurlString(node.raw) : node.raw;
}

export function suggestionsForHeader(name: string, value: string): Suggestion[] {
  const q: QueryM = { kind: "header", args: hurlString(name) };
  const out: Suggestion[] = [];
  const lower = name.toLowerCase();
  if (lower === "content-type") {
    const token = value.split(";")[0].trim();
    out.push({ label: `contains ${hurlString(token)}`, assert: mk(q, "contains", hurlString(token)) });
  }
  out.push({ label: `== ${hurlString(value)}`, assert: mk(q, "==", hurlString(value)) });
  out.push({ label: `startsWith ${hurlString(value.slice(0, 12))}`, assert: mk(q, "startsWith", hurlString(value.slice(0, 12))) });
  out.push({ label: "contains …", assert: mk(q, "contains", hurlString(value)) });
  out.push({ label: "exists", assert: mk(q, "exists") });
  out.push({ label: "not exists", assert: mk(q, "exists", "", [], true) });
  return out;
}

export function suggestionsForCookie(name: string, value: string): Suggestion[] {
  const q = (attr = ""): QueryM => ({ kind: "cookie", args: hurlString(attr ? `${name}[${attr}]` : name) });
  return [
    { label: `== ${hurlString(value)}`, assert: mk(q(), "==", hurlString(value)) },
    { label: "exists", assert: mk(q(), "exists") },
    { label: "[Secure] exists", assert: mk(q("Secure"), "exists") },
    { label: "[HttpOnly] exists", assert: mk(q("HttpOnly"), "exists") },
    { label: "[Expires] exists", assert: mk(q("Expires"), "exists") },
    { label: "[Domain] == …", assert: mk(q("Domain"), "==", '""') },
  ];
}

/** `duration < N` with N rounded up to a round number of milliseconds above the measured one. */
export function suggestionsForDuration(totalMs: number): Suggestion[] {
  const steps = [50, 100, 250, 500, 1000, 2000, 5000, 10000, 30000];
  const bound = steps.find((s) => s > totalMs) ?? (Math.floor(totalMs / 10000) + 1) * 10000;
  const q: QueryM = { kind: "duration", args: "" };
  return [
    { label: `duration < ${bound}`, assert: mk(q, "<", String(bound)) },
    { label: `duration <= ${Math.ceil(totalMs)}`, assert: mk(q, "<=", String(Math.ceil(totalMs))) },
  ];
}

export function suggestionsForStatus(status: number): Suggestion[] {
  const q: QueryM = { kind: "status", args: "" };
  return [
    { label: `status == ${status}`, assert: mk(q, "==", String(status)) },
    { label: "status >= 200", assert: mk(q, ">=", "200") },
    { label: "status < 400", assert: mk(q, "<", "400") },
  ];
}

export function suggestionsForCertificate(field: string, value: string): Suggestion[] {
  const q: QueryM = { kind: "certificate", args: hurlString(field) };
  return [
    { label: "exists", assert: mk(q, "exists") },
    { label: `== ${hurlString(value)}`, assert: mk(q, "==", hurlString(value)) },
  ];
}

/** Suggestions for the result of a query typed in the filter box (`kind` is the backend's). */
export function suggestionsForQuery(kind: "jsonpath" | "xpath", expr: string, resultKind: string, value: unknown, count: number): Suggestion[] {
  const q: QueryM = { kind, args: hurlString(expr) };
  const out: Suggestion[] = [];
  const push = (label: string, func: string, v = "", filters: FilterM[] = [], not = false) =>
    out.push({ label, assert: mk(q, func, v, filters, not) });
  switch (resultKind) {
    case "none":
      push("not exists", "exists", "", [], true);
      break;
    case "list":
      push(`count == ${count}`, "==", String(count), [{ name: "count", args: "" }]);
      push("isCollection", "isCollection");
      push("exists", "exists");
      break;
    case "nodeset":
      push(`count == ${count}`, "==", String(count), [{ name: "count", args: "" }]);
      push("exists", "exists");
      break;
    case "number": {
      const n = typeof value === "number" ? String(value) : String(value ?? "");
      push(`== ${n}`, "==", n);
      push(`> ${n}`, ">", n);
      push(`< ${n}`, "<", n);
      break;
    }
    case "bool":
      push(`== ${value}`, "==", String(value));
      break;
    case "string":
      push(`== ${hurlString(String(value))}`, "==", hurlString(String(value)));
      push("contains …", "contains", hurlString(String(value)));
      push("exists", "exists");
      break;
    case "single": {
      if (typeof value === "string") {
        push(`== ${hurlString(value)}`, "==", hurlString(value));
        push("isString", "isString");
      } else if (typeof value === "number") {
        push(`== ${value}`, "==", String(value));
        push("isNumber", "isNumber");
      } else if (typeof value === "boolean") {
        push(`== ${value}`, "==", String(value));
      } else if (value === null) {
        push("== null", "==", "null");
      } else if (Array.isArray(value)) {
        push(`count == ${value.length}`, "==", String(value.length), [{ name: "count", args: "" }]);
        push("isCollection", "isCollection");
      } else {
        push("isCollection", "isCollection");
      }
      push("exists", "exists");
      break;
    }
  }
  return out;
}

/** A capture of the same query, named by the user. */
export function captureFor(query: QueryM, name: string, filters: FilterM[] = [], redacted = false): CaptureM {
  return { name, query, filters, redacted };
}

/** The HURL text of a suggestion, for display and for "copy as line". */
export function assertText(a: AssertM): string {
  const parts = [a.query.kind, a.query.args, ...a.filters.flatMap((f) => [f.name, f.args]), a.not ? "not" : "", a.func, a.value];
  return parts.filter((p) => p.trim() !== "").join(" ");
}
