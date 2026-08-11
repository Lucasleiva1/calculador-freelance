PRAGMA foreign_keys = ON;

-- La fila de quotes conserva el estado administrativo actual. Los cálculos
-- históricos viven en quote_snapshots y nunca se actualizan ni reemplazan.
ALTER TABLE quotes ADD COLUMN notes TEXT;
ALTER TABLE quotes ADD COLUMN selected_price_kind TEXT NOT NULL DEFAULT 'recommended'
  CHECK(selected_price_kind IN ('floor','recommended','premium','custom'));
ALTER TABLE quotes ADD COLUMN selected_price_minor INTEGER
  CHECK(selected_price_minor IS NULL OR selected_price_minor >= 0);
ALTER TABLE quotes ADD COLUMN floor_total_minor INTEGER
  CHECK(floor_total_minor IS NULL OR floor_total_minor >= 0);
ALTER TABLE quotes ADD COLUMN recommended_total_minor INTEGER
  CHECK(recommended_total_minor IS NULL OR recommended_total_minor >= 0);
ALTER TABLE quotes ADD COLUMN premium_total_minor INTEGER
  CHECK(premium_total_minor IS NULL OR premium_total_minor >= 0);
ALTER TABLE quotes ADD COLUMN snapshot_revision INTEGER NOT NULL DEFAULT 0
  CHECK(snapshot_revision >= 0);
ALTER TABLE quotes ADD COLUMN saved_at TEXT;
ALTER TABLE quotes ADD COLUMN archived_at TEXT;

CREATE INDEX IF NOT EXISTS idx_quotes_history
  ON quotes(saved_at DESC, status, currency);

CREATE TABLE IF NOT EXISTS quote_snapshots (
  id TEXT PRIMARY KEY NOT NULL,
  quote_id TEXT NOT NULL REFERENCES quotes(id) ON UPDATE CASCADE ON DELETE CASCADE,
  revision INTEGER NOT NULL CHECK(revision > 0),
  schema_version INTEGER NOT NULL DEFAULT 1 CHECK(schema_version > 0),
  reason TEXT NOT NULL CHECK(reason IN ('manual_save','calculation_update','duplicate')),
  project_name TEXT NOT NULL,
  client_name TEXT NOT NULL,
  currency TEXT NOT NULL CHECK(currency IN ('ARS','USD')),
  selected_price_kind TEXT NOT NULL
    CHECK(selected_price_kind IN ('floor','recommended','premium','custom')),
  selected_price_minor INTEGER CHECK(selected_price_minor IS NULL OR selected_price_minor >= 0),
  floor_total_minor INTEGER CHECK(floor_total_minor IS NULL OR floor_total_minor >= 0),
  recommended_total_minor INTEGER CHECK(recommended_total_minor IS NULL OR recommended_total_minor >= 0),
  premium_total_minor INTEGER CHECK(premium_total_minor IS NULL OR premium_total_minor >= 0),
  total_hours_micros INTEGER NOT NULL DEFAULT 0 CHECK(total_hours_micros >= 0),
  external_costs_minor INTEGER NOT NULL DEFAULT 0 CHECK(external_costs_minor >= 0),
  effective_hourly_minor INTEGER CHECK(effective_hourly_minor IS NULL OR effective_hourly_minor >= 0),
  margin_micros INTEGER,
  snapshot_json TEXT NOT NULL CHECK(json_valid(snapshot_json)),
  created_at TEXT NOT NULL,
  UNIQUE(quote_id, revision)
);

CREATE INDEX IF NOT EXISTS idx_quote_snapshots_quote
  ON quote_snapshots(quote_id, revision DESC);

PRAGMA foreign_keys = ON;
