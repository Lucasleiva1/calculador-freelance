import type { Currency, QuoteHistoryItem, QuoteSnapshotDocument, QuoteStatus } from "./types";

export type QuoteHistorySort = "recent" | "oldest" | "price-desc" | "price-asc";
export interface QuoteHistoryFilters {
  query: string;
  status: QuoteStatus | "all";
  serviceType: string | "all";
  currency: Currency | "all";
  sort: QuoteHistorySort;
}

export const quoteStatusLabels: Record<QuoteStatus, string> = {
  draft: "Borrador",
  sent: "Enviada",
  accepted: "Aceptada",
  rejected: "Rechazada",
  archived: "Archivada",
};

export function parseQuoteSnapshot(raw: string): QuoteSnapshotDocument | null {
  try {
    const parsed = JSON.parse(raw) as QuoteSnapshotDocument;
    return parsed?.schemaVersion === 1 && Array.isArray(parsed.services) ? parsed : null;
  } catch {
    return null;
  }
}

export function filterQuoteHistory(items: QuoteHistoryItem[], filters: QuoteHistoryFilters) {
  const query = filters.query.trim().toLocaleLowerCase("es-AR");
  const filtered = items.filter((item) => {
    if (filters.status !== "all" && item.status !== filters.status) return false;
    if (filters.currency !== "all" && item.currency !== filters.currency) return false;
    if (filters.serviceType !== "all" && !item.serviceTypes.split("|").includes(filters.serviceType)) return false;
    if (query && !`${item.projectName} ${item.clientName} ${item.serviceTitles}`.toLocaleLowerCase("es-AR").includes(query)) return false;
    return true;
  });
  return [...filtered].sort((a, b) => {
    if (filters.sort === "oldest") return Date.parse(a.savedAt) - Date.parse(b.savedAt);
    if (filters.sort === "price-desc") return (b.selectedPriceMinor ?? -1) - (a.selectedPriceMinor ?? -1);
    if (filters.sort === "price-asc") return (a.selectedPriceMinor ?? Number.MAX_SAFE_INTEGER) - (b.selectedPriceMinor ?? Number.MAX_SAFE_INTEGER);
    return Date.parse(b.savedAt) - Date.parse(a.savedAt);
  });
}
