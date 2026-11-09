import { describe, expect, it } from "vitest";
import {
  IMPERIAL_UNIT_DEFAULTS,
  METRIC_UNIT_DEFAULTS,
  RESULT_UNIT_GROUPS,
  resultUnitDefaults,
  resultUnitSystemLabel
} from "./resultUnits";

describe("result unit preferences", () => {
  it("provides metric defaults", () => {
    expect(resultUnitDefaults("metric")).toEqual(METRIC_UNIT_DEFAULTS);
    expect(METRIC_UNIT_DEFAULTS["length"]).toBe("m");
    expect(METRIC_UNIT_DEFAULTS.temperature).toBe("°C");
  });

  it("provides imperial defaults", () => {
    expect(resultUnitDefaults("imperial")).toEqual(IMPERIAL_UNIT_DEFAULTS);
    expect(IMPERIAL_UNIT_DEFAULTS["length"]).toBe("ft");
    expect(IMPERIAL_UNIT_DEFAULTS.temperature).toBe("°F");
  });

  it("labels systems and exposes the customizable groups", () => {
    expect(resultUnitSystemLabel("metric")).toContain("Métrico");
    expect(resultUnitSystemLabel("imperial")).toBe("Imperial");
    expect(resultUnitSystemLabel("unknown")).toContain("Métrico");
    expect(RESULT_UNIT_GROUPS.map((group) => group.key)).toContain("length");
    expect(RESULT_UNIT_GROUPS.map((group) => group.key)).toContain("force");
  });
});
