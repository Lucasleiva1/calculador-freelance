PRAGMA foreign_keys = ON;

-- La estimación se guarda en horas para conservar las fórmulas existentes, pero
-- la interfaz permite ingresarla en horas, días o semanas de siete días.
UPDATE service_parameters
SET label='Tiempo estimado',
    description='Cargalo en horas, días o semanas; Pricing OS lo convierte a horas para calcular.',
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE parameter_key='estimatedHours';

UPDATE service_parameters
SET description='Duración total del material recibido, expresada en minutos.',
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE service_definition_id='service-video-editing' AND parameter_key='rawMinutes';

UPDATE service_parameters
SET description='Duración exacta del entregable final en formato MM:SS, desde piezas breves hasta contenido largo.',
    updated_at=strftime('%Y-%m-%dT%H:%M:%fZ','now')
WHERE service_definition_id='service-video-editing' AND parameter_key='finalDuration';
