import { describe, expect, it } from "vitest";
import {
  formatUnitDisplay,
  isUnitRun,
  nextStartsFunctionCall,
  splitResultUnit
} from "./unitText";

describe("unit text", () => {
  it("recognizes unit runs but not function names", () => {
    expect(isUnitRun("in")).toBe(true);
    expect(isUnitRun("m/s")).toBe(true);
    expect(isUnitRun("kg/m³")).toBe(true);
    expect(isUnitRun("trace")).toBe(false);
    expect(isUnitRun("min")).toBe(true);
  });

  it("detects a following function call", () => {
    expect(nextStartsFunctionCall("(2)")).toBe(true);
    expect(nextStartsFunctionCall(" (2)")).toBe(true);
    expect(nextStartsFunctionCall("+2")).toBe(false);
    expect(nextStartsFunctionCall("")).toBe(false);
  });

  it("splits a trailing unit from result text", () => {
    expect(splitResultUnit("6627.6535433071 in")).toEqual({
      value: "6627.6535433071",
      unit: " in"
    });
    expect(splitResultUnit("2.3 m")).toEqual({ value: "2.3", unit: " m" });
    expect(splitResultUnit("25°C")).toEqual({ value: "25", unit: "°C" });
    expect(splitResultUnit("1 kg/m³")).toEqual({
      value: "1",
      unit: " kg/m³"
    });
    expect(splitResultUnit("1 m/s²")).toEqual({ value: "1", unit: " m/s²" });
    expect(splitResultUnit("1 %")).toEqual({ value: "1", unit: " %" });
    expect(splitResultUnit("552.3044619423 ft")).toEqual({
      value: "552.3044619423",
      unit: " ft"
    });
    expect(splitResultUnit("30000 cm/min")).toEqual({
      value: "30000",
      unit: " cm/min"
    });
    expect(splitResultUnit("1 kg·m/s^2")).toEqual({
      value: "1",
      unit: " kg·m/s^2"
    });
  });

  it("formats unit powers with superscripts", () => {
    expect(formatUnitDisplay(" cm^2")).toBe(" cm²");
    expect(formatUnitDisplay(" kgf/cm^2")).toBe(" kgf/cm²");
    expect(formatUnitDisplay(" cm^3")).toBe(" cm³");
    expect(formatUnitDisplay(" cm^4")).toBe(" cm⁴");
    expect(formatUnitDisplay(" m/s^2")).toBe(" m/s²");
    expect(formatUnitDisplay(" m^-3")).toBe(" m⁻³");
  });

  it("keeps unitless results intact", () => {
    expect(splitResultUnit("10946/6765")).toEqual({
      value: "10946/6765",
      unit: ""
    });
    expect(splitResultUnit("1 - 2i")).toEqual({
      value: "1 - 2i",
      unit: ""
    });
    expect(splitResultUnit("2i")).toEqual({
      value: "2i",
      unit: ""
    });
    expect(splitResultUnit("0.5235987756")).toEqual({
      value: "0.5235987756",
      unit: ""
    });
  });
});
