import { describe, expect, it } from "vitest";
import {
  formatDuration,
  formatSize,
  formatTimecode,
  isEnglish,
  isSpanish,
  parseTimecode,
} from "./format";

describe("timecode helpers", () => {
  it("parses supported formats", () => {
    expect(parseTimecode("90")).toBe(90);
    expect(parseTimecode("1:30")).toBe(90);
    expect(parseTimecode("01:02:03.500")).toBeCloseTo(3723.5, 3);
    expect(parseTimecode("01:02:03,500")).toBeCloseTo(3723.5, 3);
  });

  it("rejects invalid formats", () => {
    expect(parseTimecode("")).toBeNull();
    expect(parseTimecode("abc")).toBeNull();
    expect(parseTimecode("1:90")).toBeNull();
    expect(parseTimecode("-1")).toBeNull();
  });

  it("formats and displays durations", () => {
    expect(formatTimecode(90)).toBe("00:01:30.000");
    expect(formatTimecode(3723.5)).toBe("01:02:03.500");
    expect(formatDuration(90)).toBe("1:30");
    expect(formatDuration(3723)).toBe("1:02:03");
  });

  it("formats file sizes", () => {
    expect(formatSize(1024)).toBe("1.0 KB");
    expect(formatSize(1024 * 1024 * 2.5)).toBe("2.5 MB");
  });

  it("detects English and Spanish aliases", () => {
    expect(isEnglish("en-US")).toBe(true);
    expect(isEnglish("spa")).toBe(false);
    expect(isSpanish("es-419")).toBe(true);
    expect(isSpanish("eng")).toBe(false);
  });
});
