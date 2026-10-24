import { describe, expect, it } from "vitest";
import { assertText, hurlString, jsonPathFor, regexLiteral, suggestionsForDuration, suggestionsForHeader, suggestionsForJson } from "./assert-gen";
import { parseJsonTree } from "./json-tree";

describe("jsonPathFor", () => {
  it("uses dots for identifiers and brackets otherwise", () => {
    expect(jsonPathFor(["items", 0, "id"])).toBe("$.items[0].id");
    expect(jsonPathFor(["content-type", "a b", "it's"])).toBe("$['content-type']['a b']['it\\'s']");
    expect(jsonPathFor([])).toBe("$");
  });
});

describe("hurlString", () => {
  it("escapes what HURL needs escaped", () => {
    expect(hurlString('say "hi"\\')).toBe('"say \\"hi\\"\\\\"');
    expect(hurlString("a\nb\t\u0001")).toBe('"a\\nb\\t\\u{1}"');
    expect(regexLiteral("a.b/c")).toBe("/a\\.b\\/c/");
  });
});

describe("suggestionsForJson", () => {
  const root = parseJsonTree('{"id":"550e8400-e29b-41d4-a716-446655440000","n":1.50,"ok":true,"list":[1,2],"obj":{"a":1},"when":"2024-01-02T03:04:05Z"}')!;
  const node = (k: string) => root.children!.find((c) => c.key === k)!;

  it("defaults to equality with the raw value", () => {
    expect(assertText(suggestionsForJson(node("n"))[0].assert)).toBe('jsonpath "$.n" == 1.50');
    expect(assertText(suggestionsForJson(node("ok"))[0].assert)).toBe('jsonpath "$.ok" == true');
    expect(assertText(suggestionsForJson(node("list"))[0].assert)).toBe('jsonpath "$.list" count == 2');
    expect(assertText(suggestionsForJson(node("obj"))[0].assert)).toBe('jsonpath "$.obj" exists');
  });

  it("recognises uuids and dates", () => {
    expect(suggestionsForJson(node("id")).map((s) => s.label)).toContain("isUuid");
    expect(suggestionsForJson(node("when")).map((s) => s.label)).toContain("isIsoDate");
    expect(suggestionsForJson(node("n")).map((s) => s.label)).toContain("isFloat");
  });
});

describe("other sources", () => {
  it("content-type defaults to the media type", () => {
    expect(assertText(suggestionsForHeader("Content-Type", "application/json; charset=utf-8")[0].assert)).toBe(
      'header "Content-Type" contains "application/json"',
    );
  });
  it("duration rounds up to a round bound", () => {
    expect(assertText(suggestionsForDuration(132)[0].assert)).toBe("duration < 250");
    expect(assertText(suggestionsForDuration(40000)[0].assert)).toBe("duration < 50000");
  });
});
