import { describe, expect, it } from "vitest";
import { parseEconomyImport } from "./economyImport";

describe("importación de Mi economía", () => {
  it("lee la plantilla JSON en español y conserva valores nulos fuera del formulario", () => {
    const parsed = parseEconomyImport(JSON.stringify({ moneda: "ARS", tarifaManualPorHora: 15000, horasFacturablesPorMes: 80, margenDeseadoPorcentaje: null }));
    expect(parsed.values).toEqual({ currency: "ARS", manualHourlyRate: 15000, billableHoursPerMonth: 80 });
    expect(parsed.missingFields).toContain("Ingreso mensual objetivo");
  });

  it("lee texto Markdown con etiquetas humanas y números locales", () => {
    const parsed = parseEconomyImport("| Campo | Valor |\n| --- | --- |\n| Moneda | USD |\n| Tarifa por hora | 45,50 |\n| Gastos mensuales | 1,200.75 |\n| Reserva e impuestos | 12,5% |");
    expect(parsed.values).toEqual({ currency: "USD", manualHourlyRate: 45.5, monthlyExpenses: 1200.75, reserveTaxPercent: 12.5 });
  });

  it("advierte valores no aplicables sin bloquear los demás", () => {
    const parsed = parseEconomyImport("moneda: EUR\ntarifa manual por hora: 25\nhoras facturables por mes: 0");
    expect(parsed.values.manualHourlyRate).toBe(25);
    expect(parsed.warnings).toHaveLength(2);
  });

  it("conserva la profesión declarada para impedir importarla en otro perfil", () => {
    const parsed = parseEconomyImport('{"actividad":"Programación","moneda":"ARS","tarifaManualPorHora":18000}');
    expect(parsed.values.activity).toBe("Programación");
    expect(parsed.importedFields).toContain("Actividad");
  });

  it("rechaza la guía o plantilla sin ningún valor numérico", () => {
    expect(() => parseEconomyImport('{"actividad":"Programación","moneda":"ARS","tarifaManualPorHora":"NUMERO_INVESTIGADO"}')).toThrow(/no contiene valores económicos numéricos/i);
  });
});
