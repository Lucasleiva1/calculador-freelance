import type { Currency } from "./types";

export function formatMoney(minor: number | null | undefined, currency: Currency): string {
  if (minor == null || !Number.isFinite(minor)) return "—";
  return new Intl.NumberFormat("es-AR", {
    style: "currency",
    currency,
    currencyDisplay: "code",
    minimumFractionDigits: 0,
    maximumFractionDigits: 2,
  }).format(minor / 100);
}

export function majorToMinor(value: string | number | null | undefined): number | null {
  if (value == null || value === "") return null;
  const numeric = typeof value === "number" ? value : Number(value.replace(",", "."));
  if (!Number.isFinite(numeric) || numeric < 0) return null;
  return Math.round(numeric * 100);
}

export function minorToInput(value: number | null | undefined): string {
  return value == null ? "" : String(value / 100);
}

export function convertMinor(
  amountMinor: number,
  from: Currency,
  to: Currency,
  usdToArsMicros: number | null,
): number | null {
  if (from === to) return amountMinor;
  if (!usdToArsMicros || usdToArsMicros <= 0) return null;
  const rate = usdToArsMicros / 10_000;
  return Math.round(from === "USD" ? amountMinor * rate : amountMinor / rate);
}

export function formatRate(usdToArsMicros: number | null): string {
  if (!usdToArsMicros) return "Configurar cambio";
  return new Intl.NumberFormat("es-AR", { maximumFractionDigits: 4 }).format(usdToArsMicros / 10_000);
}

