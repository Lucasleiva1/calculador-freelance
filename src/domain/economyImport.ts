import type { Currency } from "./types";

export type EconomyImportValues = {
  activity?: string;
  currency?: Currency;
  manualHourlyRate?: number;
  monthlyIncomeTarget?: number;
  monthlyExpenses?: number;
  billableHoursPerMonth?: number;
  reserveTaxPercent?: number;
  desiredMarginPercent?: number;
  defaultUrgencyPercent?: number;
  workDaysPerMonth?: number;
  vacationWeeksPerYear?: number;
};

export type EconomyImportResult = {
  values: EconomyImportValues;
  importedFields: string[];
  missingFields: string[];
  warnings: string[];
};

type Field = Exclude<keyof EconomyImportValues, "activity" | "currency">;

const labels: Record<Field, string> = {
  manualHourlyRate: "Tarifa manual por hora",
  monthlyIncomeTarget: "Ingreso mensual objetivo",
  monthlyExpenses: "Gastos mensuales",
  billableHoursPerMonth: "Horas facturables por mes",
  reserveTaxPercent: "Reserva e impuestos",
  desiredMarginPercent: "Margen deseado",
  defaultUrgencyPercent: "Urgencia predeterminada",
  workDaysPerMonth: "Días de trabajo por mes",
  vacationWeeksPerYear: "Semanas de vacaciones por año",
};

const aliases: Record<keyof EconomyImportValues, string[]> = {
  activity: ["actividad", "profesion", "profession", "activity"],
  currency: ["moneda", "currency", "divisa"],
  manualHourlyRate: ["tarifamanualporhora", "tarifaporhora", "tarifahoraria", "manualhourlyrate", "hourlyrate"],
  monthlyIncomeTarget: ["ingresomensualobjetivo", "objetivomensual", "monthlyincometarget", "monthlyincome"],
  monthlyExpenses: ["gastosmensuales", "gastos", "monthlyexpenses", "expenses"],
  billableHoursPerMonth: ["horasfacturablespormes", "horasfacturables", "horasfacturablesmes", "billablehourspermonth", "billablehours"],
  reserveTaxPercent: ["reservaimpuestosporcentaje", "reservaeimpuestos", "reservaimpuestos", "reserva", "taxreservepercent", "reservetaxpercent"],
  desiredMarginPercent: ["margendeseadoporcentaje", "margendeseado", "margen", "desiredmarginpercent", "desiredmargin"],
  defaultUrgencyPercent: ["urgenciapredeterminadaporcentaje", "urgenciapredeterminada", "urgencia", "defaulturgencypercent", "defaulturgency"],
  workDaysPerMonth: ["diasdetrabajopormes", "diasdetrabajo", "workdayspermonth", "workdays"],
  vacationWeeksPerYear: ["semanasvacacionesporanio", "semanasdevacaciones", "vacaciones", "vacationweeksperyear", "vacationweeks"],
};

function normalized(value: string) {
  return value.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase().replace(/[^a-z0-9]/g, "");
}

function localizedNumber(value: unknown): number | undefined {
  if (typeof value === "number") return Number.isFinite(value) ? value : undefined;
  if (typeof value !== "string") return undefined;
  const compact = value.trim().replace(/[^0-9,.-]/g, "");
  if (!compact || compact === "-" || compact === "." || compact === ",") return undefined;
  const comma = compact.lastIndexOf(",");
  const dot = compact.lastIndexOf(".");
  let canonical = compact;
  if (comma >= 0 && dot >= 0) canonical = comma > dot ? compact.replace(/\./g, "").replace(",", ".") : compact.replace(/,/g, "");
  else if (comma >= 0) canonical = compact.length - comma - 1 <= 2 ? compact.replace(",", ".") : compact.replace(/,/g, "");
  else if (dot >= 0 && compact.length - dot - 1 > 2) canonical = compact.replace(/\./g, "");
  const number = Number(canonical);
  return Number.isFinite(number) ? number : undefined;
}

function recordFromJson(text: string): Record<string, unknown> | null {
  const candidates = [text.trim(), ...[...text.matchAll(/```(?:json)?\s*([\s\S]*?)```/gi)].map((match) => match[1])];
  for (const candidate of candidates) {
    try {
      const parsed = JSON.parse(candidate) as unknown;
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
        const root = parsed as Record<string, unknown>;
        const nested = [root.economia, root.economy, root.miEconomia].find((value) => value && typeof value === "object" && !Array.isArray(value));
        return (nested ?? root) as Record<string, unknown>;
      }
    } catch { /* try the next shape */ }
  }
  return null;
}

function recordFromText(text: string): Record<string, unknown> {
  const record: Record<string, unknown> = {};
  for (const rawLine of text.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || /^\|?\s*[-:| ]+\|\s*[-:| ]+/.test(line)) continue;
    const cells = line.split("|").map((cell) => cell.trim()).filter(Boolean);
    const pair = cells.length >= 2 ? [cells[0], cells[1]] : line.replace(/^[-*•]\s*/, "").split(/\s*[:=]\s*/, 2);
    if (pair.length === 2 && pair[0].length <= 80 && pair[1]) record[pair[0]] = pair[1];
  }
  return record;
}

function readValue(record: Record<string, unknown>, field: keyof EconomyImportValues) {
  const entries = Object.entries(record);
  const match = entries.find(([key]) => aliases[field].includes(normalized(key)));
  return match?.[1];
}

export function parseEconomyImport(text: string): EconomyImportResult {
  const record = recordFromJson(text) ?? recordFromText(text);
  const values: EconomyImportValues = {};
  const importedFields: string[] = [];
  const warnings: string[] = [];
  const rawActivity = readValue(record, "activity");
  if (rawActivity != null && String(rawActivity).trim()) {
    values.activity = String(rawActivity).trim();
    importedFields.push("Actividad");
  }
  const rawCurrency = readValue(record, "currency");
  if (rawCurrency != null) {
    const currency = String(rawCurrency).trim().toUpperCase();
    if (currency === "ARS" || currency === "USD") { values.currency = currency; importedFields.push("Moneda"); }
    else warnings.push("La moneda debe ser ARS o USD; no se aplicó ese valor.");
  }
  (Object.keys(labels) as Field[]).forEach((field) => {
    const raw = readValue(record, field);
    if (raw == null || raw === "") return;
    const value = localizedNumber(raw);
    if (value == null || value < 0) { warnings.push(`${labels[field]} no contiene un número válido; no se aplicó.`); return; }
    if (field === "billableHoursPerMonth" && value === 0) { warnings.push("Las horas facturables deben ser mayores que cero; no se aplicaron."); return; }
    values[field] = value;
    importedFields.push(labels[field]);
  });
  const missingFields = (Object.keys(labels) as Field[]).filter((field) => values[field] == null).map((field) => labels[field]);
  if (missingFields.length === Object.keys(labels).length) {
    throw new Error("Este archivo no contiene valores económicos numéricos. Si es la guía para IA o una plantilla con textos NUMERO_, primero pedile a la IA el JSON final completo e importá esa respuesta.");
  }
  return { values, importedFields, missingFields, warnings };
}

export function importNumberInput(value: number | undefined) {
  return value == null ? undefined : String(value);
}
