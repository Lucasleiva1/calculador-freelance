import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const updaterMock = vi.hoisted(() => ({ currentVersion: vi.fn(), check: vi.fn() }));
vi.mock("../../services/updater", () => ({ appUpdater: updaterMock }));

import { UpdateSettings } from "./UpdateSettings";

describe("UpdateSettings", () => {
  beforeEach(() => {
    updaterMock.currentVersion.mockReset().mockResolvedValue("0.1.0");
    updaterMock.check.mockReset();
  });

  it("informa cuando la instalación ya está actualizada", async () => {
    updaterMock.check.mockResolvedValue(null);
    render(<UpdateSettings onBeforeInstall={vi.fn().mockResolvedValue(true)} />);
    fireEvent.click(screen.getByRole("button", { name: /buscar actualizaciones/i }));
    await waitFor(() => expect(screen.getByText("Ya tenés la versión estable más reciente.")).toBeInTheDocument());
    expect(screen.getByText("v0.1.0")).toBeInTheDocument();
  });

  it("guarda el borrador, instala el asset firmado y muestra el progreso", async () => {
    const install = vi.fn(async (onProgress: (value: { downloadedBytes: number; totalBytes: number; finished: boolean }) => void) => {
      onProgress({ downloadedBytes: 50, totalBytes: 100, finished: false });
    });
    updaterMock.check.mockResolvedValue({ currentVersion: "0.1.0", version: "0.2.0", date: null, notes: "Mejoras", install, dispose: vi.fn() });
    const onBeforeInstall = vi.fn().mockResolvedValue(true);
    render(<UpdateSettings onBeforeInstall={onBeforeInstall} />);
    fireEvent.click(screen.getByRole("button", { name: /buscar actualizaciones/i }));
    await screen.findByText("La versión 0.2.0 está lista para descargar e instalar.");
    fireEvent.click(screen.getByRole("button", { name: /descargar e instalar/i }));
    await waitFor(() => expect(install).toHaveBeenCalledTimes(1));
    expect(onBeforeInstall).toHaveBeenCalledTimes(1);
    expect(screen.getByLabelText("Descarga 50%")).toBeInTheDocument();
  });

  it("no instala si quedan cambios sin guardar", async () => {
    const install = vi.fn();
    updaterMock.check.mockResolvedValue({ currentVersion: "0.1.0", version: "0.2.0", date: null, notes: null, install, dispose: vi.fn() });
    render(<UpdateSettings onBeforeInstall={vi.fn().mockResolvedValue(false)} />);
    fireEvent.click(screen.getByRole("button", { name: /buscar actualizaciones/i }));
    await screen.findByText("La versión 0.2.0 está lista para descargar e instalar.");
    fireEvent.click(screen.getByRole("button", { name: /descargar e instalar/i }));
    await screen.findByText(/no se pudo guardar todo/i);
    expect(install).not.toHaveBeenCalled();
  });
});
