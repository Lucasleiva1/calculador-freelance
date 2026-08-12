import { describe, expect, it } from "vitest";
import { canonicalizePrintDesignValues, estimatePrintDesignEffort, normalizePrintDesignEffort } from "./printDesign";

describe("estimación de esfuerzo para diseño de estampas", () => {
  it("estima horas cuando el alcance tiene tipo y complejidad", () => {
    const estimate = estimatePrintDesignEffort({
      mainWorkType: "design-from-scratch",
      complexity: "high",
      inputQuality: "bad",
      additionalOperations: ["add-text", "vectorize", "prepare-dtf"],
    });
    expect(estimate?.hours).toBeGreaterThan(10);
  });

  it("guarda la estimación automática para que las fuentes horarias puedan normalizarse", () => {
    const normalized = normalizePrintDesignEffort({ mainWorkType: "vector-corrected", complexity: "medium" });
    expect(normalized.estimatedHoursMode).toBe("automatic");
    expect(normalized.estimatedHours).toBeGreaterThan(0);
  });

  it("nunca reemplaza un tiempo manual existente", () => {
    const normalized = normalizePrintDesignEffort({ mainWorkType: "design-from-scratch", complexity: "premium", estimatedHours: 2.5 });
    expect(normalized.estimatedHoursMode).toBe("manual");
    expect(normalized.estimatedHours).toBe(2.5);
  });

  it("recalcula si el modo ya es automático", () => {
    const normalized = normalizePrintDesignEffort({ mainWorkType: "redraw-full", complexity: "high", estimatedHours: 1, estimatedHoursMode: "automatic" });
    expect(normalized.estimatedHours).toBeGreaterThan(1);
  });

  it("hace participar las tareas detalladas que representan trabajo", () => {
    const base = estimatePrintDesignEffort({ mainWorkType: "design-from-scratch", complexity: "medium" });
    const detailed = estimatePrintDesignEffort({ mainWorkType: "design-from-scratch", complexity: "medium", aiActions: ["multiple-tests", "photoshop-retouch"], printActions: ["png", "review-contrast"], editableDelivery: "psd" });
    expect(detailed!.hours).toBeGreaterThan(base!.hours);
  });

  it("unifica opciones antiguas duplicadas en sus campos canónicos", () => {
    const values = canonicalizePrintDesignValues({ additionalOperations: ["prepare-dtf", "apply-halftone", "deliver-editable", "vectorize"] });
    expect(values.additionalOperations).toEqual(["vectorize"]);
    expect(values.printOutput).toEqual(["dtf"]);
    expect(values.halftoneLevel).toBe("simple");
    expect(values.editableDelivery).toBe("other");
  });

  it("completa urgencia y origen coherentes cuando el alcance ya los determina", () => {
    const fromScratch = canonicalizePrintDesignValues({ mainWorkType: "design-from-scratch" });
    expect(fromScratch.urgency).toBe("normal");
    expect(fromScratch.designOrigin).toBe("from-scratch");
    const restoration = canonicalizePrintDesignValues({ mainWorkType: "restore-image" });
    expect(restoration.designOrigin).toBe("reference-to-redo");
  });
});
