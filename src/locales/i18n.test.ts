import { describe, expect, it } from "vitest";
import en from "./en.json";
import es from "./es.json";

function placeholders(value: string): Set<string> {
  return new Set(
    [...value.matchAll(/\{\{\s*([a-zA-Z0-9_]+)\s*\}\}/g)].map((match) => match[1]),
  );
}

describe("translation catalogs", () => {
  it("have the same keys", () => {
    expect(Object.keys(es).sort()).toEqual(Object.keys(en).sort());
  });

  it("keep identical interpolation placeholders", () => {
    const mismatches = Object.keys(en).filter((key) => {
      const expected = placeholders(en[key as keyof typeof en]);
      const actual = placeholders(es[key as keyof typeof es]);
      return (
        expected.size !== actual.size ||
        [...expected].some((placeholder) => !actual.has(placeholder))
      );
    });
    expect(mismatches).toEqual([]);
  });
});
