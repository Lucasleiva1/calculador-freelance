PRAGMA foreign_keys = ON;

-- Diseño de estampas comparte la infraestructura de investigación, pero usa
-- referencias propias de diseño gráfico. Nunca reutiliza precios de video o
-- programación.
INSERT OR IGNORE INTO market_sources (
  id,name,base_url,source_type,regions_json,supported_services_json,priority,enabled,
  usage_mode,acquisition_mode,cooldown_hours,notes,is_system_source,system_key,
  default_data_json,purpose,data_contribution,app_benefit,participates_in_suggestions,
  automation_status,current_status,adapter_key,business_source_type,market_country,
  source_currency,source_updated_at,classification_origin,created_at,updated_at
) VALUES
(
  'source-ardg-print-design','ARDG · Tarifario de diseño Argentina',
  'https://ardg.ar/tarifario/','professional_tariff',
  '["AR"]','["print-design"]',12,1,'market_price','auto_http',24,
  'Tarifario oficial de la Asociación Rosarina de Diseño Gráfico, actualizado en julio de 2026.',1,
  'ardg-print-design',
  json_object(
    'name','ARDG · Tarifario de diseño Argentina',
    'baseUrl','https://ardg.ar/tarifario/','sourceType','professional_tariff','regionsJson','["AR"]',
    'supportedServicesJson','["print-design"]','priority',12,'enabled',1,
    'usageMode','market_price','acquisitionMode','auto_http','cooldownHours',24,
    'purpose','Valores profesionales de diseño gráfico para Argentina.',
    'dataContribution','Valor hora y precio específico para remera en ARS por categoría de cliente.',
    'appBenefit','Calcula el precio de mercado argentino para diseño de estampas con una fuente profesional primaria.',
    'participatesInSuggestions',1,'automationStatus','APPROVED','adapterKey','ardg-print-design',
    'currentStatus','READY','lastError',NULL,'businessSourceType','market',
    'marketCountry','Argentina','sourceCurrency','ARS','sourceUpdatedAt','2026-07-01'
  ),
  'Valores profesionales de diseño gráfico para Argentina.',
  'Valor hora y precio específico para remera en ARS por categoría de cliente.',
  'Calcula el precio de mercado argentino para diseño de estampas con una fuente profesional primaria.',
  1,'APPROVED','READY','ardg-print-design','market','Argentina','ARS','2026-07-01','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
),
(
  'source-twine-print-design','Twine · diseño gráfico freelance',
  'https://www.twine.net/blog/freelance-graphic-designer-hourly-rates/','rate_benchmark',
  '["GLOBAL","INTERNATIONAL"]','["print-design"]',18,1,'market_price','auto_http',24,
  'Bandas horarias de diseño gráfico freelance publicadas en noviembre de 2025.',1,
  'twine-print-design',
  json_object(
    'name','Twine · diseño gráfico freelance',
    'baseUrl','https://www.twine.net/blog/freelance-graphic-designer-hourly-rates/',
    'sourceType','rate_benchmark','regionsJson','["GLOBAL","INTERNATIONAL"]',
    'supportedServicesJson','["print-design"]','priority',18,'enabled',1,
    'usageMode','market_price','acquisitionMode','auto_http','cooldownHours',24,
    'purpose','Bandas internacionales de diseño gráfico freelance por experiencia.',
    'dataContribution','Rangos entry, mid y senior en USD por hora, fecha y enlace.',
    'appBenefit','Calcula una referencia internacional específica de diseño gráfico.',
    'participatesInSuggestions',1,'automationStatus','APPROVED',
    'adapterKey','twine-graphic-design','currentStatus','READY','lastError',NULL,
    'businessSourceType','market','marketCountry','Global','sourceCurrency','USD',
    'sourceUpdatedAt','2025-11-21'
  ),
  'Bandas internacionales de diseño gráfico freelance por experiencia.',
  'Rangos entry, mid y senior en USD por hora, fecha y enlace.',
  'Calcula una referencia internacional específica de diseño gráfico.',
  1,'APPROVED','READY','twine-graphic-design','market','Global','USD','2025-11-21','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
),
(
  'source-freelancerateiq-print-design','FreelanceRateIQ · diseño gráfico 2026',
  'https://freelancerateiq.com/blog/freelance-graphic-design-rates','rate_benchmark',
  '["GLOBAL","INTERNATIONAL"]','["print-design"]',19,1,'market_price','auto_http',24,
  'Bandas horarias de diseño gráfico freelance publicadas en abril de 2026.',1,
  'freelancerateiq-print-design',
  json_object(
    'name','FreelanceRateIQ · diseño gráfico 2026',
    'baseUrl','https://freelancerateiq.com/blog/freelance-graphic-design-rates',
    'sourceType','rate_benchmark','regionsJson','["GLOBAL","INTERNATIONAL"]',
    'supportedServicesJson','["print-design"]','priority',19,'enabled',1,
    'usageMode','market_price','acquisitionMode','auto_http','cooldownHours',24,
    'purpose','Benchmark 2026 de diseño gráfico freelance por experiencia.',
    'dataContribution','Rangos entry, junior, mid, senior y expert en USD por hora.',
    'appBenefit','Contrasta el precio internacional con una segunda fuente específica.',
    'participatesInSuggestions',1,'automationStatus','APPROVED',
    'adapterKey','freelancerateiq-graphic-design','currentStatus','READY','lastError',NULL,
    'businessSourceType','market','marketCountry','Global','sourceCurrency','USD',
    'sourceUpdatedAt','2026-04-13'
  ),
  'Benchmark 2026 de diseño gráfico freelance por experiencia.',
  'Rangos entry, junior, mid, senior y expert en USD por hora.',
  'Contrasta el precio internacional con una segunda fuente específica.',
  1,'APPROVED','READY','freelancerateiq-graphic-design','market','Global','USD','2026-04-13','automatic',
  strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')
);

INSERT OR REPLACE INTO pricing_engine_sources (
  engine_id,source_id,role,preference,participates_in_suggestions,
  match_score_micros,explanation,assigned_by,created_at,updated_at
) VALUES
('engine-print-design','source-ardg-print-design','reference','preferred',1,990000,
 'Tarifario profesional oficial de diseño gráfico con valor específico para remera en Argentina.','automatic',
 strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('engine-print-design','source-twine-print-design','reference','preferred',1,950000,
 'Benchmark internacional de diseño gráfico freelance por experiencia.','automatic',
 strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('engine-print-design','source-freelancerateiq-print-design','reference','preferred',1,950000,
 'Segundo benchmark internacional actualizado de diseño gráfico freelance.','automatic',
 strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'));

-- Respaldo auditable de la edición oficial vigente. La app intenta actualizar
-- ARDG por HTTP; si el dominio argentino tiene una caída temporal de DNS, estos
-- valores fechados evitan que el usuario quede sin precio local.
INSERT OR IGNORE INTO market_observations (
  id,source_id,origin,service_type,subservice,category,region,country,currency,
  price_type,unit,price_min_minor,price_max_minor,price_value_minor,
  original_value_text,experience_level,client_tier,source_type,source_url,
  published_at,retrieved_at,parser_version,confidence,comparison_eligibility,
  exclusion_reason,raw_fingerprint,evidence_snippet,notes,created_at
) VALUES
('obs-ardg-print-hour-a','source-ardg-print-design','AUTO','print-design','Diseño gráfico para estampas','ARDG · valor hora','AR','Argentina','ARS','HOURLY','por hora',NULL,NULL,4200000,'ARS 42.000 por hora · Cliente A','Semi Senior','A','professional_rate_card','https://ardg.ar/tarifario/','2026-07-01',strftime('%Y-%m-%dT%H:%M:%fZ','now'),'ardg-print-design/seed-2026-07','HIGH','ELIGIBLE',NULL,'ardg-print-hour-a-2026-07','ARDG · valor hora · Cliente A: ARS 42.000','Tarifario oficial ARDG, edición julio de 2026.',strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('obs-ardg-print-hour-b','source-ardg-print-design','AUTO','print-design','Diseño gráfico para estampas','ARDG · valor hora','AR','Argentina','ARS','HOURLY','por hora',NULL,NULL,3000000,'ARS 30.000 por hora · Cliente B','Semi Senior','B','professional_rate_card','https://ardg.ar/tarifario/','2026-07-01',strftime('%Y-%m-%dT%H:%M:%fZ','now'),'ardg-print-design/seed-2026-07','HIGH','ELIGIBLE',NULL,'ardg-print-hour-b-2026-07','ARDG · valor hora · Cliente B: ARS 30.000','Tarifario oficial ARDG, edición julio de 2026.',strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('obs-ardg-print-hour-c','source-ardg-print-design','AUTO','print-design','Diseño gráfico para estampas','ARDG · valor hora','AR','Argentina','ARS','HOURLY','por hora',NULL,NULL,2400000,'ARS 24.000 por hora · Cliente C','Semi Senior','C','professional_rate_card','https://ardg.ar/tarifario/','2026-07-01',strftime('%Y-%m-%dT%H:%M:%fZ','now'),'ardg-print-design/seed-2026-07','HIGH','ELIGIBLE',NULL,'ardg-print-hour-c-2026-07','ARDG · valor hora · Cliente C: ARS 24.000','Tarifario oficial ARDG, edición julio de 2026.',strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('obs-ardg-print-remera-a','source-ardg-print-design','AUTO','print-design','Diseño para remera','ARDG · Promocionales · Remera','AR','Argentina','ARS','PROJECT','por proyecto',NULL,NULL,25200000,'ARS 252.000 por diseño de remera · Cliente A','Semi Senior','A','professional_rate_card','https://ardg.ar/tarifario/','2026-07-01',strftime('%Y-%m-%dT%H:%M:%fZ','now'),'ardg-print-design/seed-2026-07','HIGH','ELIGIBLE',NULL,'ardg-print-remera-a-2026-07','ARDG · Remera · Cliente A: ARS 252.000','Tarifario oficial ARDG, edición julio de 2026.',strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('obs-ardg-print-remera-b','source-ardg-print-design','AUTO','print-design','Diseño para remera','ARDG · Promocionales · Remera','AR','Argentina','ARS','PROJECT','por proyecto',NULL,NULL,18000000,'ARS 180.000 por diseño de remera · Cliente B','Semi Senior','B','professional_rate_card','https://ardg.ar/tarifario/','2026-07-01',strftime('%Y-%m-%dT%H:%M:%fZ','now'),'ardg-print-design/seed-2026-07','HIGH','ELIGIBLE',NULL,'ardg-print-remera-b-2026-07','ARDG · Remera · Cliente B: ARS 180.000','Tarifario oficial ARDG, edición julio de 2026.',strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('obs-ardg-print-remera-c','source-ardg-print-design','AUTO','print-design','Diseño para remera','ARDG · Promocionales · Remera','AR','Argentina','ARS','PROJECT','por proyecto',NULL,NULL,14400000,'ARS 144.000 por diseño de remera · Cliente C','Semi Senior','C','professional_rate_card','https://ardg.ar/tarifario/','2026-07-01',strftime('%Y-%m-%dT%H:%M:%fZ','now'),'ardg-print-design/seed-2026-07','HIGH','ELIGIBLE',NULL,'ardg-print-remera-c-2026-07','ARDG · Remera · Cliente C: ARS 144.000','Tarifario oficial ARDG, edición julio de 2026.',strftime('%Y-%m-%dT%H:%M:%fZ','now'));

UPDATE market_sources
SET supported_services_json='["video-editing","programming","print-design"]',
    default_data_json=json_set(default_data_json,'$.supportedServicesJson','["video-editing","programming","print-design"]'),
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE system_key='bcra';
