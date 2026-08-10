import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ResultInspector } from "./ResultInspector";

describe("ResultInspector", () => {
  it("presenta el estado vacío sin precios ficticios", () => {
    render(<ResultInspector currency="USD" activeServiceId={null} suggestionsEnabled result={{ services: [], totalMinor: null, totalHours: 0, externalCostsMinor: 0, effectiveHourlyMinor: null, unpricedCount: 0, isPartial: false }} />);
    expect(screen.getByText("Agregá un servicio para comenzar la cotización.")).toBeInTheDocument();
    expect(screen.getAllByText("—").length).toBeGreaterThan(0);
    expect(screen.getByText("Sin extracción automática")).toBeInTheDocument();
  });
});
