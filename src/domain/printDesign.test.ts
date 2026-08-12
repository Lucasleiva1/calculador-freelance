import { describe, expect, it } from "vitest";
import { canonicalizePrintDesignValues, estimatePrintDesignEffort, normalizePrintDesignEffort, suggestedPrintDesignComplexity } from "./printDesign";

describe("Diseño de estampas v3", () => {
  it("clasifica puntajes y fuerza complejo al crear desde cero", () => {
    expect(suggestedPrintDesignComplexity({ hasReference: true, materialType: "low-quality", workTasks: ["remove-background"] })).toEqual({ complexity: "basic", score: 2 });
    expect(suggestedPrintDesignComplexity({ hasReference: true, workTasks: ["reconstruct-image", "adapt-composition"] })).toEqual({ complexity: "intermediate", score: 5 });
    expect(suggestedPrintDesignComplexity({ hasReference: false, workTasks: ["design-from-scratch"] })).toEqual({ complexity: "complex", score: 4 });
  });

  it("estima desde base, tareas, conceptualización, multiplicador y cuartos de hora", () => {
    const estimate = estimatePrintDesignEffort({ hasReference: false, workTasks: ["design-from-scratch"] });
    expect(estimate.hours).toBe(8.75);
    expect(estimate.complexity).toBe("complex");
  });

  it("respeta el override exacto hasta restaurar el modo automático", () => {
    const manual = normalizePrintDesignEffort({ hasReference: false, workTasks: ["design-from-scratch"], estimatedHours: 2.5, estimatedHoursMode: "manual" });
    expect(manual.estimatedHours).toBe(2.5);
    const automatic = normalizePrintDesignEffort({ ...manual, estimatedHoursMode: "automatic" });
    expect(automatic.estimatedHours).toBe(8.75);
  });

  it("migra tareas, origen, calidad, impresión y editables anteriores", () => {
    const migrated = canonicalizePrintDesignValues({
      designOrigin: "existing-design", mainWorkType: "vector-corrected", inputQuality: "bad",
      additionalOperations: ["remove-background", "apply-halftone", "prepare-dtf"], editableDelivery: "psd",
    });
    expect(migrated.hasReference).toBe(true);
    expect(migrated.materialType).toBe("low-quality");
    expect(migrated.printSystem).toBe("dtf");
    expect(migrated.workTasks).toEqual(expect.arrayContaining(["vectorize-simple", "remove-background", "halftone"]));
    expect(migrated.deliveryExtras).toEqual(["psd"]);
  });

  it("limpia valores condicionales y marca contradicciones antiguas para reselección", () => {
    const cleaned = canonicalizePrintDesignValues({ hasReference: false, materialType: "screenshot", printSystem: "dtf", sublimationFitsA4: false, productType: "shirt", otherProduct: "Taza" });
    expect(cleaned.materialType).toBeUndefined();
    expect(cleaned.sublimationFitsA4).toBeUndefined();
    expect(cleaned.otherProduct).toBeUndefined();
    const ambiguous = canonicalizePrintDesignValues({ designOrigin: "from-scratch", mainWorkType: "background-complex" });
    expect(ambiguous.hasReference).toBeUndefined();
    expect(ambiguous.designOrigin).toBeUndefined();
    expect(ambiguous.mainWorkType).toBeUndefined();
  });
});
