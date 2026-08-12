import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ProjectResult } from "../../domain/quote";
import type { Workspace } from "../../domain/types";
import { SaveQuoteModal } from "./SaveQuoteModal";

const workspace: Workspace = {
  project: { id: "p", clientId: "c", clientName: "Cliente", name: "Campaña", currency: "USD", marketScope: "international", status: "active", totalMinor: 120_000, unpricedCount: 0, updatedAt: "" },
  quote: { id: "q", projectId: "p", version: 1, status: "draft", currency: "USD", notes: null, selectedPriceKind: "recommended", selectedPriceMinor: null, floorTotalMinor: null, recommendedTotalMinor: null, premiumTotalMinor: null, snapshotRevision: 0, savedAt: null, archivedAt: null, createdAt: "", updatedAt: "" },
  services: [],
};
const result: ProjectResult = { services: [], totalMinor: 120_000, totalHours: 10, externalCostsMinor: 5_000, effectiveHourlyMinor: 11_500, marginMicros: 250_000, pricingTiers: { floorMinor: 90_000, recommendedMinor: 120_000, premiumMinor: 145_000 }, unpricedCount: 0, isPartial: false };

describe("guardado de cotización", () => {
  it("explica el snapshot y conserva el precio final elegido", async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    render(<SaveQuoteModal workspace={workspace} result={result} onClose={() => undefined} onSave={save} />);
    expect(screen.getByText(/el proyecto ya se guarda automáticamente/i)).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText(/Precio final elegido/i), { target: { value: "1300" } });
    fireEvent.click(screen.getByRole("button", { name: /guardar cotización/i }));
    await waitFor(() => expect(save).toHaveBeenCalledWith(expect.objectContaining({ selectedPriceKind: "custom", selectedPriceMinor: 130_000, recommendedTotalMinor: 120_000, totalHoursMicros: 10_000_000 })));
  });
});
