PRAGMA foreign_keys = ON;

-- El contrato de Estampas v2 reemplaza el formulario técnico, pero conserva
-- sus filas desactivadas para que snapshots y auditorías sigan siendo legibles.
UPDATE service_definitions
SET version=2,
    description='Cotización simple de diseño de estampas por alcance, horas y tres precios independientes.',
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE id='service-print-design';

UPDATE pricing_engines
SET description='Diseño de estampas con precio sostenible, mercado argentino e internacional separados.',
    classification_version=2,
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE id='engine-print-design';

UPDATE service_parameters SET enabled=0,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE service_definition_id='service-print-design';
UPDATE parameter_options SET enabled=0,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE parameter_id IN (SELECT id FROM service_parameters WHERE service_definition_id='service-print-design');
UPDATE pricing_rules SET enabled=0,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE service_definition_id='service-print-design';

-- Complejidad y horas conservan sus claves e identificadores estables.
UPDATE service_parameters SET name='Complejidad',label='Complejidad',description='Complejidad sugerida por tareas o reemplazada manualmente.',required=1,sort_order=100,enabled=1,suggestion_enabled=0,ui_managed=1,version=2,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id='pd-complexity';
UPDATE service_parameters SET name='Horas estimadas',label='Horas estimadas',description='Horas núcleo numéricas utilizadas por los tres precios.',required=1,sort_order=140,enabled=1,suggestion_enabled=0,ui_managed=1,version=2,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id='pd-hours';
UPDATE parameter_options SET label='Básica',value='basic',sort_order=10,enabled=1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id='pdc-basic';
UPDATE parameter_options SET label='Intermedia',value='intermediate',sort_order=20,enabled=1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id='pdc-medium';
UPDATE parameter_options SET label='Compleja',value='complex',sort_order=30,enabled=1,updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id='pdc-high';

INSERT OR IGNORE INTO service_parameters (
  id,service_definition_id,parameter_key,name,label,parameter_type,description,
  required,sort_order,enabled,suggestion_enabled,is_system,ui_managed,version,created_at,updated_at
) VALUES
('pd-v3-reference','service-print-design','hasReference','Referencia','¿Existe una referencia?','boolean','Define si se trabaja desde material recibido o desde cero.',1,10,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-material','service-print-design','materialType','Material recibido','Material recibido','single_select','Calidad y función del material aportado.',0,20,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-client','service-print-design','clientTier','Categoría de cliente','Categoría de cliente','single_select','Categoría ARDG C, B o A.',1,30,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-product','service-print-design','productType','Producto','Producto','single_select','Producto donde se aplicará el diseño.',1,40,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-other-product','service-print-design','otherProduct','Otro producto','Otro producto','text','Nombre del producto cuando no existe una opción específica.',0,50,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-tone','service-print-design','garmentTone','Tono','Tono de la prenda o soporte','single_select','Claro, oscuro o ambos.',1,60,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-system','service-print-design','printSystem','Sistema','Sistema de impresión','single_select','DTF, sublimación o sólo diseño.',1,70,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-a4','service-print-design','sublimationFitsA4','Formato A4','¿Entra en una hoja A4?','boolean','Sólo aplica a sublimación.',0,80,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-tasks','service-print-design','workTasks','Tareas','Trabajo necesario','multi_select','Tareas profesionales concretas del encargo.',1,90,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-complexity-mode','service-print-design','complexityMode','Modo de complejidad','Modo de complejidad','single_select','Automático o manual.',1,110,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-hours-mode','service-print-design','estimatedHoursMode','Modo de horas','Modo de horas','single_select','Automático o manual.',1,120,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-effort','service-print-design','effortAmount','Cantidad de tiempo','Cantidad de tiempo','number','Valor visible en horas o días.',0,130,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-effort-unit','service-print-design','effortUnit','Unidad','Unidad de tiempo','single_select','Horas o días.',1,150,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-day','service-print-design','hoursPerDay','Jornada','Horas por jornada','number','Conversión exacta de días a horas.',1,160,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-delivery','service-print-design','deliveryExtras','Entrega','Entregables adicionales','multi_select','Editables, versiones o tamaños adicionales.',0,170,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pd-v3-price','service-print-design','priceSelection','Precio elegido','Precio elegido','text','Snapshot de la elección explícita entre sostenible, Argentina e internacional.',0,180,1,0,1,1,2,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'));

INSERT OR IGNORE INTO parameter_options (id,parameter_id,label,value,sort_order,enabled,created_at,updated_at) VALUES
('pdv3-m-ready','pd-v3-material','Archivo usable','ready',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-m-low','pd-v3-material','Baja calidad','low-quality',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-m-shot','pd-v3-material','Captura','screenshot',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-m-ref','pd-v3-material','Sólo referencia','reference-only',40,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-c-small','pd-v3-client','Pequeño / C','small',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-c-medium','pd-v3-client','Mediano / B','medium',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-c-large','pd-v3-client','Grande / A','large',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-p-shirt','pd-v3-product','Remera','shirt',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-p-hoodie','pd-v3-product','Buzo','hoodie',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-p-sock','pd-v3-product','Media','sock',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-p-other','pd-v3-product','Otro','other',40,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-t-light','pd-v3-tone','Claro','light',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-t-dark','pd-v3-tone','Oscuro','dark',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-t-both','pd-v3-tone','Ambos','both',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-s-dtf','pd-v3-system','DTF','dtf',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-s-sub','pd-v3-system','Sublimación','sublimation',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-s-design','pd-v3-system','Sólo diseño','design-only',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-w-bg','pd-v3-tasks','Quitar fondo','remove-background',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-w-quality','pd-v3-tasks','Mejorar calidad / resolución','improve-quality',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-w-rebuild','pd-v3-tasks','Reconstruir o completar imagen','reconstruct-image',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-w-vector','pd-v3-tasks','Vectorizar texto o gráfico simple','vectorize-simple',40,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-w-opt','pd-v3-tasks','Ajustar y optimizar imagen','optimize-image',50,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-w-comp','pd-v3-tasks','Adaptar composición para estampa','adapt-composition',60,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-w-grunge','pd-v3-tasks','Crear bordes / grunge / integrar imagen','grunge-borders',70,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-w-half','pd-v3-tasks','Aplicar semitono','halftone',80,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-w-ai','pd-v3-tasks','Generar o reconstruir elementos con IA','ai-elements',90,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-w-scratch','pd-v3-tasks','Crear diseño desde cero','design-from-scratch',100,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-mode-auto','pd-v3-complexity-mode','Automático','automatic',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-mode-manual','pd-v3-complexity-mode','Manual','manual',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-hours-auto','pd-v3-hours-mode','Automático','automatic',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-hours-manual','pd-v3-hours-mode','Manual','manual',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-unit-hours','pd-v3-effort-unit','Horas','hours',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-unit-days','pd-v3-effort-unit','Días','days',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-d-psd','pd-v3-delivery','PSD editable','psd',10,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-d-ai','pd-v3-delivery','AI / vector editable','ai-vector',20,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-d-versions','pd-v3-delivery','Versiones adicionales','extra-versions',30,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('pdv3-d-sizes','pd-v3-delivery','Tamaños o adaptaciones adicionales','extra-sizes',40,1,strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'));

-- Sólo los borradores vivos cambian de esquema. Los snapshots e históricos no se tocan.
UPDATE quote_services
SET configuration_version=3,
    configuration_json=json_set(configuration_json,'$.schemaVersion',3),
    calculated_subtotal_minor=NULL,suggested_subtotal_minor=NULL,final_subtotal_minor=NULL,
    has_override=0,manual_subtotal_minor=NULL,manual_reason=NULL,pricing_snapshot_json=NULL,
    service_definition_version=2,
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE service_type='print-design' AND deleted_at IS NULL
  AND quote_id IN (SELECT id FROM quotes WHERE status='draft');

-- ARDG agrega Uniforme para adaptación; el diseño original conserva Remera.
INSERT OR IGNORE INTO market_observations (
  id,source_id,origin,service_type,subservice,category,region,country,currency,price_type,unit,
  price_min_minor,price_max_minor,price_value_minor,original_value_text,experience_level,client_tier,
  source_type,source_url,published_at,retrieved_at,parser_version,confidence,comparison_eligibility,
  exclusion_reason,raw_fingerprint,evidence_snippet,notes,created_at
) VALUES
('obs-ardg-print-uniforme-a','source-ardg-print-design','AUTO','print-design','Adaptación de marca para indumentaria','ARDG · Promocionales · Uniforme','AR','Argentina','ARS','PROJECT','por proyecto',NULL,NULL,12600000,'ARS 126.000 por uniforme · Cliente A',NULL,'A','professional_rate_card','https://ardg.ar/tarifario/','2026-07-01',strftime('%Y-%m-%dT%H:%M:%fZ','now'),'ardg-print-design/seed-2026-07-v2','HIGH','ELIGIBLE',NULL,'ardg-print-uniforme-a-2026-07','ARDG · Uniforme · Cliente A: ARS 126.000','Tarifario oficial ARDG, julio de 2026.',strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('obs-ardg-print-uniforme-b','source-ardg-print-design','AUTO','print-design','Adaptación de marca para indumentaria','ARDG · Promocionales · Uniforme','AR','Argentina','ARS','PROJECT','por proyecto',NULL,NULL,9000000,'ARS 90.000 por uniforme · Cliente B',NULL,'B','professional_rate_card','https://ardg.ar/tarifario/','2026-07-01',strftime('%Y-%m-%dT%H:%M:%fZ','now'),'ardg-print-design/seed-2026-07-v2','HIGH','ELIGIBLE',NULL,'ardg-print-uniforme-b-2026-07','ARDG · Uniforme · Cliente B: ARS 90.000','Tarifario oficial ARDG, julio de 2026.',strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('obs-ardg-print-uniforme-c','source-ardg-print-design','AUTO','print-design','Adaptación de marca para indumentaria','ARDG · Promocionales · Uniforme','AR','Argentina','ARS','PROJECT','por proyecto',NULL,NULL,7200000,'ARS 72.000 por uniforme · Cliente C',NULL,'C','professional_rate_card','https://ardg.ar/tarifario/','2026-07-01',strftime('%Y-%m-%dT%H:%M:%fZ','now'),'ardg-print-design/seed-2026-07-v2','HIGH','ELIGIBLE',NULL,'ardg-print-uniforme-c-2026-07','ARDG · Uniforme · Cliente C: ARS 72.000','Tarifario oficial ARDG, julio de 2026.',strftime('%Y-%m-%dT%H:%M:%fZ','now'));

INSERT OR IGNORE INTO market_sources (
  id,name,base_url,source_type,regions_json,supported_services_json,priority,enabled,usage_mode,
  acquisition_mode,cooldown_hours,notes,is_system_source,system_key,default_data_json,purpose,
  data_contribution,app_benefit,participates_in_suggestions,automation_status,current_status,
  adapter_key,business_source_type,market_country,source_currency,source_updated_at,
  classification_origin,created_at,updated_at
) VALUES
('source-upwork-print-design','Upwork · diseño gráfico','https://www.upwork.com/hire/graphic-designers/cost/','freelance_marketplace','["GLOBAL","INTERNATIONAL"]','["print-design"]',17,1,'market_price','auto_http',24,'Tarifas horarias freelance de diseño gráfico.',1,'upwork-print-design',NULL,'Referencia internacional independiente.','Rangos horarios por experiencia en USD.','Contrasta el precio internacional sin mezclarlo con el sostenible.',1,'APPROVED','READY','upwork','market','Global','USD','2026-01-01','automatic',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('source-adg-cordoba-method','ADG Córdoba · metodología','https://www.adg.com.ar/mi-cuenta/tarifario/','professional_tariff','["AR"]','["print-design"]',50,1,'context_only','manual',NULL,'Respaldo metodológico sin valores automatizables verificables.',1,'adg-cordoba-method',NULL,'Contexto metodológico argentino.','No aporta importes automáticos.','Documenta un criterio profesional alternativo sin contaminar la mediana.',0,'MANUAL_ONLY','MANUAL','generic','methodology','Argentina','ARS',NULL,'manual',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'));

INSERT OR REPLACE INTO pricing_engine_sources (
 engine_id,source_id,role,preference,participates_in_suggestions,match_score_micros,explanation,assigned_by,created_at,updated_at
) VALUES
('engine-print-design','source-upwork-print-design','reference','preferred',1,950000,'Benchmark internacional trazable de diseño gráfico.','automatic',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now')),
('engine-print-design','source-adg-cordoba-method','context','available',0,700000,'Respaldo metodológico sin valores automatizables.','manual',strftime('%Y-%m-%dT%H:%M:%fZ','now'),strftime('%Y-%m-%dT%H:%M:%fZ','now'));

UPDATE market_sources
SET base_url='https://www.twine.net/pricing',
    default_data_json=json_set(default_data_json,'$.baseUrl','https://www.twine.net/pricing'),
    data_contribution='Bandas entry, mid y senior en USD por hora, con fecha y enlace trazable.',
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE id='source-twine-print-design';
UPDATE market_sources
SET data_contribution='Valor hora, Uniforme para adaptación y Remera para diseño original, por categoría A/B/C.',
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE id='source-ardg-print-design';
