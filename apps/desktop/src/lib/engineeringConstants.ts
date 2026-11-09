import constantsData from "./engineering-constants.json";

export type EngineeringConstant = {
  name: string;
  value: number;
  unit: string;
  dimension: [number, number, number, number, number, number, number];
  description: string;
  category: string;
};

export const ENGINEERING_CONSTANTS =
  constantsData as EngineeringConstant[];

export const ENGINEERING_CONSTANT_NAMES = new Set(
  ENGINEERING_CONSTANTS.map((constant) => constant.name)
);

export function findEngineeringConstant(
  name: string
): EngineeringConstant | undefined {
  return ENGINEERING_CONSTANTS.find((constant) => constant.name === name);
}
