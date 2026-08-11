export type EffortUnit = "hours" | "days" | "weeks";

export const DEFAULT_HOURS_PER_DAY = 8;

export function effortToHours(
  amount: number | null,
  unit: EffortUnit,
  hoursPerDay = DEFAULT_HOURS_PER_DAY,
): number | null {
  if (amount == null || !Number.isFinite(amount)) return null;
  const safeHoursPerDay = Number.isFinite(hoursPerDay) && hoursPerDay > 0 ? hoursPerDay : DEFAULT_HOURS_PER_DAY;
  if (unit === "days") return amount * safeHoursPerDay;
  if (unit === "weeks") return amount * 7 * safeHoursPerDay;
  return amount;
}

export function hoursToEffort(
  hours: number | null,
  unit: EffortUnit,
  hoursPerDay = DEFAULT_HOURS_PER_DAY,
): number | null {
  if (hours == null || !Number.isFinite(hours)) return null;
  const safeHoursPerDay = Number.isFinite(hoursPerDay) && hoursPerDay > 0 ? hoursPerDay : DEFAULT_HOURS_PER_DAY;
  if (unit === "days") return hours / safeHoursPerDay;
  if (unit === "weeks") return hours / (7 * safeHoursPerDay);
  return hours;
}

export function effortUnitLabel(unit: EffortUnit, amount: number) {
  if (unit === "weeks") return amount === 1 ? "semana" : "semanas";
  if (unit === "days") return amount === 1 ? "día" : "días";
  return amount === 1 ? "hora" : "horas";
}
