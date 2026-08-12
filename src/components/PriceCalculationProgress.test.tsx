import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PriceCalculationProgress, type PriceCalculationProgressState } from "./PriceCalculationProgress";

const running: PriceCalculationProgressState = {
  mode: "calculate",
  phase: "market",
  jobId: "job-1",
  localReady: true,
  completedSources: 2,
  totalSources: 4,
};

describe("PriceCalculationProgress", () => {
  it("muestra el avance secuencial de los tres precios", () => {
    render(<PriceCalculationProgress state={running} onCancel={vi.fn()} onDismiss={vi.fn()} />);

    expect(screen.getByRole("dialog", { name: /calculando tus 3 precios/i })).toBeInTheDocument();
    expect(screen.getByText("Precio local / sostenible")).toBeInTheDocument();
    expect(screen.getByText("Precio de mercado Argentina")).toBeInTheDocument();
    expect(screen.getByText("Precio internacional")).toBeInTheDocument();
    expect(screen.getByText(/2 de 4/)).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "55");
  });

  it("explica que el precio local pendiente no bloquea los automáticos", () => {
    render(<PriceCalculationProgress state={{ ...running, localReady: false }} onCancel={vi.fn()} onDismiss={vi.fn()} />);
    expect(screen.getByText(/los precios automáticos continúan/i)).toBeInTheDocument();
    expect(screen.getByText("Pendiente")).toBeInTheDocument();
  });
});
