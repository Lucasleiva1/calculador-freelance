import { describe, expect, it } from "vitest";
import { effortToHours, hoursToEffort } from "./effort";

describe("effort units", () => {
  it("converts days using the selected working day", () => {
    expect(effortToHours(3, "days", 8)).toBe(24);
    expect(effortToHours(3, "days", 12)).toBe(36);
  });

  it("treats a week as seven days", () => {
    expect(effortToHours(1, "weeks", 8)).toBe(56);
    expect(hoursToEffort(56, "weeks", 8)).toBe(1);
  });

  it("preserves direct hourly estimates", () => {
    expect(effortToHours(12, "hours", 8)).toBe(12);
    expect(hoursToEffort(12, "hours", 8)).toBe(12);
  });
});
