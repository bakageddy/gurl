// A JSON reader for the response tree. Unlike JSON.parse it keeps every number's source text
// (`1.50` stays `1.50`, 64-bit ids keep their digits), which is what a HURL assert must compare
// against, and it gives every node its JSONPath.

export type JsonKind = "object" | "array" | "string" | "number" | "boolean" | "null";

export interface JsonNode {
  /** Property name or array index; null for the root. */
  key: string | number | null;
  path: (string | number)[];
  kind: JsonKind;
  /** Scalars: the value as text (strings decoded, numbers verbatim from the source). */
  raw: string;
  children?: JsonNode[];
  /** Number of children for containers, 0 otherwise. */
  count: number;
}

const MAX_DEPTH = 512;

class Reader {
  i = 0;
  constructor(readonly s: string) {}

  ws() {
    const s = this.s;
    while (this.i < s.length) {
      const c = s.charCodeAt(this.i);
      if (c === 32 || c === 9 || c === 10 || c === 13) this.i++;
      else break;
    }
  }

  fail(what: string): never {
    throw new SyntaxError(`${what} at offset ${this.i}`);
  }

  value(key: string | number | null, path: (string | number)[], depth: number): JsonNode {
    if (depth > MAX_DEPTH) this.fail("too deeply nested");
    this.ws();
    const c = this.s[this.i];
    if (c === "{") return this.object(key, path, depth);
    if (c === "[") return this.array(key, path, depth);
    if (c === '"') return { key, path, kind: "string", raw: this.string(), count: 0 };
    if (c === "t" && this.s.startsWith("true", this.i)) return this.lit(key, path, "true", "boolean");
    if (c === "f" && this.s.startsWith("false", this.i)) return this.lit(key, path, "false", "boolean");
    if (c === "n" && this.s.startsWith("null", this.i)) return this.lit(key, path, "null", "null");
    const m = /^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?/.exec(this.s.slice(this.i, this.i + 400));
    if (m) {
      this.i += m[0].length;
      return { key, path, kind: "number", raw: m[0], count: 0 };
    }
    this.fail("unexpected character");
  }

  private lit(key: string | number | null, path: (string | number)[], word: string, kind: JsonKind): JsonNode {
    this.i += word.length;
    return { key, path, kind, raw: word, count: 0 };
  }

  private string(): string {
    const s = this.s;
    const start = this.i;
    this.i++; // opening quote
    while (this.i < s.length) {
      const c = s[this.i];
      if (c === "\\") this.i += 2;
      else if (c === '"') {
        this.i++;
        return JSON.parse(s.slice(start, this.i)) as string;
      } else this.i++;
    }
    this.fail("unterminated string");
  }

  private object(key: string | number | null, path: (string | number)[], depth: number): JsonNode {
    this.i++;
    const children: JsonNode[] = [];
    this.ws();
    if (this.s[this.i] === "}") {
      this.i++;
      return { key, path, kind: "object", raw: "", children, count: 0 };
    }
    for (;;) {
      this.ws();
      if (this.s[this.i] !== '"') this.fail("expected a property name");
      const name = this.string();
      this.ws();
      if (this.s[this.i] !== ":") this.fail("expected ':'");
      this.i++;
      children.push(this.value(name, [...path, name], depth + 1));
      this.ws();
      const c = this.s[this.i];
      if (c === ",") {
        this.i++;
        continue;
      }
      if (c === "}") {
        this.i++;
        return { key, path, kind: "object", raw: "", children, count: children.length };
      }
      this.fail("expected ',' or '}'");
    }
  }

  private array(key: string | number | null, path: (string | number)[], depth: number): JsonNode {
    this.i++;
    const children: JsonNode[] = [];
    this.ws();
    if (this.s[this.i] === "]") {
      this.i++;
      return { key, path, kind: "array", raw: "", children, count: 0 };
    }
    for (;;) {
      const idx = children.length;
      children.push(this.value(idx, [...path, idx], depth + 1));
      this.ws();
      const c = this.s[this.i];
      if (c === ",") {
        this.i++;
        continue;
      }
      if (c === "]") {
        this.i++;
        return { key, path, kind: "array", raw: "", children, count: children.length };
      }
      this.fail("expected ',' or ']'");
    }
  }
}

/** The tree for `text`, or null when it isn't JSON. */
export function parseJsonTree(text: string): JsonNode | null {
  try {
    const r = new Reader(text);
    const root = r.value(null, [], 0);
    r.ws();
    if (r.i !== text.length) return null;
    return root;
  } catch {
    return null;
  }
}

/** Counts every node in the tree (used to cap "expand all"). */
export function nodeCount(node: JsonNode): number {
  let n = 1;
  for (const c of node.children ?? []) n += nodeCount(c);
  return n;
}

/** Paths of nodes whose key or scalar value contains `needle` (case-insensitive), with a cap. */
export function searchTree(root: JsonNode, needle: string, cap = 500): JsonNode[] {
  const q = needle.toLowerCase();
  const out: JsonNode[] = [];
  const walk = (n: JsonNode) => {
    if (out.length >= cap) return;
    const key = n.key === null ? "" : String(n.key).toLowerCase();
    if (key.includes(q) || (n.kind !== "object" && n.kind !== "array" && n.raw.toLowerCase().includes(q))) out.push(n);
    for (const c of n.children ?? []) walk(c);
  };
  if (q) walk(root);
  return out;
}
