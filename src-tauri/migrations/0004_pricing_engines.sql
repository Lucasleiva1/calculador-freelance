PRAGMA foreign_keys = ON;

ALTER TABLE app_settings ADD COLUMN help_mode TEXT NOT NULL DEFAULT 'guided'
  CHECK(help_mode IN ('guided','compact','off'));
ALTER TABLE app_settings ADD COLUMN local_ai_enabled INTEGER NOT NULL DEFAULT 0
  CHECK(local_ai_enabled IN (0,1));
ALTER TABLE app_settings ADD COLUMN ollama_base_url TEXT NOT NULL DEFAULT 'http://127.0.0.1:11434';
ALTER TABLE app_settings ADD COLUMN ollama_model TEXT;
ALTER TABLE app_settings ADD COLUMN ai_auto_apply_high_confidence INTEGER NOT NULL DEFAULT 0
  CHECK(ai_auto_apply_high_confidence IN (0,1));

ALTER TABLE market_sources ADD COLUMN business_source_type TEXT NOT NULL DEFAULT 'market';
ALTER TABLE market_sources ADD COLUMN market_country TEXT;
ALTER TABLE market_sources ADD COLUMN source_currency TEXT;
ALTER TABLE market_sources ADD COLUMN source_updated_at TEXT;
ALTER TABLE market_sources ADD COLUMN classification_origin TEXT NOT NULL DEFAULT 'automatic'
  CHECK(classification_origin IN ('automatic','ai_assisted','manual'));
ALTER TABLE market_sources ADD COLUMN classification_json TEXT;

CREATE TABLE IF NOT EXISTS engine_categories (
  id TEXT PRIMARY KEY NOT NULL,
  parent_id TEXT REFERENCES engine_categories(id) ON UPDATE CASCADE ON DELETE RESTRICT,
  slug TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL CHECK(length(trim(name)) > 0),
  engine_type TEXT CHECK(engine_type IS NULL OR engine_type IN ('service','product','hybrid')),
  description TEXT,
  is_system INTEGER NOT NULL DEFAULT 0 CHECK(is_system IN (0,1)),
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO engine_categories
  (id, parent_id, slug, name, engine_type, description, is_system, sort_order, created_at, updated_at)
VALUES
  ('category-services', NULL, 'services', 'Servicios', 'service', 'Trabajos profesionales o digitales.', 1, 10, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('category-audiovisual', 'category-services', 'audiovisual', 'Audiovisual', 'service', 'Video, fotografía, motion y producción audiovisual.', 1, 11, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('category-design', 'category-services', 'design', 'Diseño', 'service', 'Diseño gráfico, web, de producto e indumentaria.', 1, 12, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('category-technology', 'category-services', 'technology', 'Tecnología', 'service', 'Software, automatización y servicios digitales.', 1, 13, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('category-products', NULL, 'products', 'Productos', 'product', 'Bienes físicos producidos o revendidos.', 1, 20, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('category-apparel', 'category-products', 'apparel', 'Indumentaria', 'product', 'Prendas, accesorios y productos textiles.', 1, 21, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('category-food', 'category-products', 'food', 'Alimentos', 'product', 'Alimentos, bebidas y comidas preparadas.', 1, 22, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('category-home', 'category-products', 'home', 'Hogar y objetos', 'product', 'Objetos de uso cotidiano y artículos para el hogar.', 1, 23, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('category-custom-products', 'category-products', 'custom-products', 'Productos personalizados', 'product', 'Productos físicos con personalización o producción a pedido.', 1, 24, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('category-hybrid', NULL, 'hybrid', 'Híbridos', 'hybrid', 'Propuestas que combinan servicio y producto.', 1, 30, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('category-design-production', 'category-hybrid', 'design-production', 'Diseño y producción', 'hybrid', 'Diseño profesional combinado con fabricación o entrega física.', 1, 31, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'));

CREATE TABLE IF NOT EXISTS pricing_engines (
  id TEXT PRIMARY KEY NOT NULL,
  engine_key TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL CHECK(length(trim(name)) > 0),
  description TEXT,
  engine_type TEXT NOT NULL CHECK(engine_type IN ('service','product','hybrid')),
  category_id TEXT REFERENCES engine_categories(id) ON UPDATE CASCADE ON DELETE SET NULL,
  calculator_key TEXT NOT NULL CHECK(calculator_key IN ('professional-service-v1','physical-product-v1','hybrid-v1','unconfigured')),
  service_definition_id TEXT UNIQUE REFERENCES service_definitions(id) ON UPDATE CASCADE ON DELETE SET NULL,
  unit_kind TEXT NOT NULL DEFAULT 'project',
  tags_json TEXT NOT NULL DEFAULT '[]',
  status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','active','archived')),
  classification_origin TEXT NOT NULL DEFAULT 'automatic' CHECK(classification_origin IN ('automatic','ai_assisted','manual')),
  classification_confidence_micros INTEGER CHECK(classification_confidence_micros IS NULL OR (classification_confidence_micros >= 0 AND classification_confidence_micros <= 1000000)),
  classification_explanation TEXT,
  classification_version INTEGER NOT NULL DEFAULT 1 CHECK(classification_version > 0),
  is_system INTEGER NOT NULL DEFAULT 0 CHECK(is_system IN (0,1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  archived_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_pricing_engines_category ON pricing_engines(category_id, status, name);
CREATE INDEX IF NOT EXISTS idx_pricing_engines_type ON pricing_engines(engine_type, status, name);

INSERT OR IGNORE INTO pricing_engines
  (id, engine_key, name, description, engine_type, category_id, calculator_key,
   service_definition_id, unit_kind, tags_json, status, classification_origin,
   classification_confidence_micros, classification_explanation, is_system, created_at, updated_at)
SELECT 'engine-video-editing', 'video-editing', name, description, 'service',
       'category-audiovisual', 'professional-service-v1', id, 'project',
       '["video","edición","audiovisual"]', 'active', 'automatic', 1000000,
       'Motor audiovisual incorporado por el sistema.', 1, created_at, updated_at
FROM service_definitions WHERE service_type='video-editing';

INSERT OR IGNORE INTO pricing_engines
  (id, engine_key, name, description, engine_type, category_id, calculator_key,
   service_definition_id, unit_kind, tags_json, status, classification_origin,
   classification_confidence_micros, classification_explanation, is_system, created_at, updated_at)
SELECT 'engine-programming', 'programming', name, description, 'service',
       'category-technology', 'professional-service-v1', id, 'project',
       '["programación","software","tecnología"]', 'active', 'automatic', 1000000,
       'Motor tecnológico incorporado por el sistema.', 1, created_at, updated_at
FROM service_definitions WHERE service_type='programming';

CREATE TABLE IF NOT EXISTS pricing_engine_sources (
  engine_id TEXT NOT NULL REFERENCES pricing_engines(id) ON UPDATE CASCADE ON DELETE CASCADE,
  source_id TEXT NOT NULL REFERENCES market_sources(id) ON UPDATE CASCADE ON DELETE CASCADE,
  role TEXT NOT NULL CHECK(role IN ('reference','cost_input','context')),
  preference TEXT NOT NULL DEFAULT 'available' CHECK(preference IN ('preferred','available','excluded')),
  participates_in_suggestions INTEGER NOT NULL DEFAULT 0 CHECK(participates_in_suggestions IN (0,1)),
  match_score_micros INTEGER NOT NULL DEFAULT 0 CHECK(match_score_micros >= 0 AND match_score_micros <= 1000000),
  explanation TEXT,
  assigned_by TEXT NOT NULL DEFAULT 'automatic' CHECK(assigned_by IN ('automatic','ai_assisted','manual')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(engine_id, source_id)
);

CREATE INDEX IF NOT EXISTS idx_pricing_engine_sources_source ON pricing_engine_sources(source_id, preference);

INSERT OR IGNORE INTO pricing_engine_sources
  (engine_id, source_id, role, preference, participates_in_suggestions,
   match_score_micros, explanation, assigned_by, created_at, updated_at)
SELECT pe.id, ms.id,
       CASE ms.usage_mode
         WHEN 'market_price' THEN 'reference'
         WHEN 'currency' THEN 'cost_input'
         ELSE 'context' END,
       'available', ms.participates_in_suggestions, 800000,
       'Asignación migrada desde la compatibilidad declarada por la fuente.',
       'automatic', strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now')
FROM market_sources ms
JOIN json_each(ms.supported_services_json) supported
JOIN pricing_engines pe ON pe.engine_key = supported.value;

CREATE TABLE IF NOT EXISTS classification_aliases (
  id TEXT PRIMARY KEY NOT NULL,
  normalized_term TEXT NOT NULL UNIQUE,
  engine_type TEXT NOT NULL CHECK(engine_type IN ('service','product','hybrid')),
  category_id TEXT REFERENCES engine_categories(id) ON UPDATE CASCADE ON DELETE SET NULL,
  tags_json TEXT NOT NULL DEFAULT '[]',
  origin TEXT NOT NULL DEFAULT 'system' CHECK(origin IN ('system','user')),
  use_count INTEGER NOT NULL DEFAULT 0 CHECK(use_count >= 0),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO classification_aliases
  (id, normalized_term, engine_type, category_id, tags_json, origin, created_at, updated_at)
VALUES
  ('alias-remeras','remeras','product','category-apparel','["indumentaria","remeras"]','system',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('alias-medias','medias','product','category-apparel','["indumentaria","medias"]','system',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('alias-empanadas','empanadas','product','category-food','["alimentos","comidas preparadas","empanadas"]','system',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('alias-termos','termos','product','category-home','["hogar","termos"]','system',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('alias-video','edicion de video','service','category-audiovisual','["video","edición"]','system',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
  ('alias-programacion','programacion','service','category-technology','["programación","software"]','system',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'));

CREATE TABLE IF NOT EXISTS classification_runs (
  id TEXT PRIMARY KEY NOT NULL,
  subject_type TEXT NOT NULL CHECK(subject_type IN ('engine','source')),
  subject_id TEXT,
  input_json TEXT NOT NULL,
  automatic_proposal_json TEXT NOT NULL,
  ai_proposal_json TEXT,
  final_proposal_json TEXT NOT NULL,
  ai_used INTEGER NOT NULL DEFAULT 0 CHECK(ai_used IN (0,1)),
  ai_model TEXT,
  status TEXT NOT NULL CHECK(status IN ('success','fallback','error')),
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_classification_runs_subject ON classification_runs(subject_type, subject_id, created_at DESC);
