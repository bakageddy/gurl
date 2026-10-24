import { describe, expect, it } from "vitest";
import { nodeCount, parseJsonTree, searchTree } from "./json-tree";

describe("parseJsonTree", () => {
  it("keeps number text and decodes strings", () => {
    const root = parseJsonTree('{"a":1.50,"big":12345678901234567890,"s":"x\\ny","t":true,"n":null,"l":[1,{"k":[]}]}')!;
    expect(root.kind).toBe("object");
    const byKey = Object.fromEntries(root.children!.map((c) => [c.key, c]));
    expect(byKey.a.raw).toBe("1.50");
    expect(byKey.big.raw).toBe("12345678901234567890");
    expect(byKey.s.raw).toBe("x\ny");
    expect(byKey.t.kind).toBe("boolean");
    expect(byKey.n.kind).toBe("null");
    expect(byKey.l.count).toBe(2);
    expect(byKey.l.children![1].children![0].path).toEqual(["l", 1, "k"]);
    expect(nodeCount(root)).toBe(10);
  });

  it("rejects non-JSON and trailing garbage", () => {
    expect(parseJsonTree("not json")).toBeNull();
    expect(parseJsonTree("{} x")).toBeNull();
    expect(parseJsonTree("")).toBeNull();
    expect(parseJsonTree("  [ ] ")!.kind).toBe("array");
  });

  it("searches keys and values", () => {
    const root = parseJsonTree('{"name":"gurl","items":[{"id":"abc"},{"id":"ABD"}]}')!;
    expect(searchTree(root, "ab").map((n) => n.path.join("."))).toEqual(["items.0.id", "items.1.id"]);
    expect(searchTree(root, "name").length).toBe(1);
    expect(searchTree(root, "").length).toBe(0);
  });
});
