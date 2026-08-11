PRAGMA foreign_keys = ON;

-- Datos que se pueden compartir con un cliente. Se mantienen separados de los
-- snapshots internos, que contienen los cálculos y las fuentes de evidencia.
CREATE TABLE IF NOT EXISTS professional_profile (
  id INTEGER PRIMARY KEY NOT NULL CHECK(id = 1),
  display_name TEXT NOT NULL DEFAULT '',
  business_name TEXT,
  email TEXT,
  phone TEXT,
  website TEXT,
  location TEXT,
  logo_path TEXT,
  default_currency TEXT NOT NULL DEFAULT 'USD' CHECK(default_currency IN ('ARS','USD')),
  default_quote_validity_days INTEGER CHECK(default_quote_validity_days IN (7,15,30)),
  default_client_terms TEXT,
  document_theme TEXT NOT NULL DEFAULT 'light' CHECK(document_theme IN ('light','dark')),
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO professional_profile (id, updated_at)
VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

CREATE TABLE IF NOT EXISTS quote_number_counters (
  year INTEGER PRIMARY KEY NOT NULL,
  next_sequence INTEGER NOT NULL CHECK(next_sequence > 0)
);

ALTER TABLE quotes ADD COLUMN quote_number TEXT;

CREATE TABLE IF NOT EXISTS quote_client_details (
  quote_id TEXT PRIMARY KEY NOT NULL REFERENCES quotes(id) ON UPDATE CASCADE ON DELETE CASCADE,
  presentation_mode TEXT NOT NULL DEFAULT 'itemized' CHECK(presentation_mode IN ('global','itemized')),
  scope TEXT,
  revisions TEXT,
  estimated_timeline TEXT,
  client_notes TEXT,
  valid_until TEXT,
  service_descriptions_json TEXT NOT NULL DEFAULT '{}' CHECK(json_valid(service_descriptions_json)),
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_quotes_number ON quotes(quote_number);
