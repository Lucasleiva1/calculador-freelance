PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS clients (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL CHECK(length(trim(name)) > 0),
  company TEXT,
  email TEXT,
  whatsapp TEXT,
  country TEXT,
  notes TEXT,
  status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'archived')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
  id TEXT PRIMARY KEY NOT NULL,
  client_id TEXT NOT NULL REFERENCES clients(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  name TEXT NOT NULL CHECK(length(trim(name)) > 0),
  currency TEXT NOT NULL CHECK(currency IN ('ARS', 'USD')),
  market_scope TEXT CHECK(market_scope IN ('argentina', 'international', 'both')),
  status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'archived')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_projects_client ON projects(client_id);
CREATE INDEX IF NOT EXISTS idx_projects_updated ON projects(updated_at DESC);

CREATE TABLE IF NOT EXISTS quotes (
  id TEXT PRIMARY KEY NOT NULL,
  project_id TEXT NOT NULL REFERENCES projects(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  version INTEGER NOT NULL CHECK(version > 0),
  status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'sent', 'accepted', 'rejected', 'archived')),
  currency TEXT NOT NULL CHECK(currency IN ('ARS', 'USD')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(project_id, version)
);

CREATE TABLE IF NOT EXISTS quote_services (
  id TEXT PRIMARY KEY NOT NULL,
  quote_id TEXT NOT NULL REFERENCES quotes(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  service_type TEXT NOT NULL,
  title TEXT NOT NULL,
  sort_order INTEGER NOT NULL CHECK(sort_order >= 0),
  configuration_version INTEGER NOT NULL DEFAULT 1 CHECK(configuration_version > 0),
  configuration_json TEXT NOT NULL,
  calculated_subtotal_minor INTEGER CHECK(calculated_subtotal_minor IS NULL OR calculated_subtotal_minor >= 0),
  manual_subtotal_minor INTEGER CHECK(manual_subtotal_minor IS NULL OR manual_subtotal_minor >= 0),
  manual_reason TEXT,
  row_revision INTEGER NOT NULL DEFAULT 0 CHECK(row_revision >= 0),
  deleted_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_quote_services_quote ON quote_services(quote_id, sort_order);

CREATE TABLE IF NOT EXISTS service_presets (
  id TEXT PRIMARY KEY NOT NULL,
  service_type TEXT NOT NULL,
  name TEXT NOT NULL CHECK(length(trim(name)) > 0),
  origin TEXT NOT NULL CHECK(origin IN ('system', 'user')),
  system_key TEXT UNIQUE,
  configuration_version INTEGER NOT NULL DEFAULT 1,
  configuration_json TEXT NOT NULL,
  default_configuration_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_presets_service ON service_presets(service_type, origin, name);

CREATE TABLE IF NOT EXISTS app_settings (
  id INTEGER PRIMARY KEY NOT NULL CHECK(id = 1),
  theme TEXT NOT NULL DEFAULT 'warm' CHECK(theme IN ('warm', 'dark')),
  hourly_rate_ars_minor INTEGER CHECK(hourly_rate_ars_minor IS NULL OR hourly_rate_ars_minor >= 0),
  hourly_rate_usd_minor INTEGER CHECK(hourly_rate_usd_minor IS NULL OR hourly_rate_usd_minor >= 0),
  usd_to_ars_micros INTEGER CHECK(usd_to_ars_micros IS NULL OR usd_to_ars_micros > 0),
  active_project_id TEXT REFERENCES projects(id) ON UPDATE CASCADE ON DELETE SET NULL,
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO app_settings (id, theme, updated_at)
VALUES (1, 'warm', strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

INSERT OR IGNORE INTO service_presets (
  id, service_type, name, origin, system_key, configuration_version,
  configuration_json, default_configuration_json, created_at, updated_at
) VALUES
('preset-reel-standard', 'video-editing', 'Reel / Short — Estándar', 'system', 'reel-standard', 1,
 '{"pieceType":"reel-short","quantity":1,"resolution":"1080p","editingLevel":"professional","revisions":2,"formats":["9:16"],"color":"basic","audio":"music-effects","subtitles":"designed","videoAi":"none","voiceAi":false,"soundAi":false,"backgroundRemoval":false,"motion":"none","broll":"client","additionalVersions":0}',
 '{"pieceType":"reel-short","quantity":1,"resolution":"1080p","editingLevel":"professional","revisions":2,"formats":["9:16"],"color":"basic","audio":"music-effects","subtitles":"designed","videoAi":"none","voiceAi":false,"soundAi":false,"backgroundRemoval":false,"motion":"none","broll":"client","additionalVersions":0}',
 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('preset-youtube-standard', 'video-editing', 'YouTube — Estándar', 'system', 'youtube-standard', 1,
 '{"pieceType":"youtube","quantity":1,"resolution":"1080p","editingLevel":"professional","revisions":2,"formats":["16:9"],"color":"basic","audio":"music-effects","subtitles":"standard","videoAi":"none","voiceAi":false,"soundAi":false,"backgroundRemoval":false,"motion":"basic","broll":"client","additionalVersions":0}',
 '{"pieceType":"youtube","quantity":1,"resolution":"1080p","editingLevel":"professional","revisions":2,"formats":["16:9"],"color":"basic","audio":"music-effects","subtitles":"standard","videoAi":"none","voiceAi":false,"soundAi":false,"backgroundRemoval":false,"motion":"basic","broll":"client","additionalVersions":0}',
 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('preset-ad-ai', 'video-editing', 'Publicidad con IA', 'system', 'ad-ai', 1,
 '{"pieceType":"advertising","quantity":1,"resolution":"1080p","editingLevel":"advanced","revisions":2,"formats":["16:9","9:16"],"color":"look","audio":"music-effects","subtitles":"designed","videoAi":"important","voiceAi":false,"soundAi":true,"backgroundRemoval":true,"motion":"ai-assisted","broll":"simple","additionalVersions":0}',
 '{"pieceType":"advertising","quantity":1,"resolution":"1080p","editingLevel":"advanced","revisions":2,"formats":["16:9","9:16"],"color":"look","audio":"music-effects","subtitles":"designed","videoAi":"important","voiceAi":false,"soundAi":true,"backgroundRemoval":true,"motion":"ai-assisted","broll":"simple","additionalVersions":0}',
 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('preset-institutional', 'video-editing', 'Institucional', 'system', 'institutional', 1,
 '{"pieceType":"institutional","quantity":1,"resolution":"1080p","editingLevel":"professional","revisions":2,"formats":["16:9"],"color":"look","audio":"cleanup","subtitles":"standard","videoAi":"none","voiceAi":false,"soundAi":false,"backgroundRemoval":false,"motion":"basic","broll":"client","additionalVersions":0}',
 '{"pieceType":"institutional","quantity":1,"resolution":"1080p","editingLevel":"professional","revisions":2,"formats":["16:9"],"color":"look","audio":"cleanup","subtitles":"standard","videoAi":"none","voiceAi":false,"soundAi":false,"backgroundRemoval":false,"motion":"basic","broll":"client","additionalVersions":0}',
 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
('preset-podcast', 'video-editing', 'Podcast', 'system', 'podcast', 1,
 '{"pieceType":"podcast","quantity":1,"resolution":"1080p","editingLevel":"professional","revisions":2,"formats":["16:9"],"color":"basic","audio":"cleanup","subtitles":"standard","videoAi":"none","voiceAi":false,"soundAi":false,"backgroundRemoval":false,"motion":"none","broll":"client","additionalVersions":0}',
 '{"pieceType":"podcast","quantity":1,"resolution":"1080p","editingLevel":"professional","revisions":2,"formats":["16:9"],"color":"basic","audio":"cleanup","subtitles":"standard","videoAi":"none","voiceAi":false,"soundAi":false,"backgroundRemoval":false,"motion":"none","broll":"client","additionalVersions":0}',
 strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

