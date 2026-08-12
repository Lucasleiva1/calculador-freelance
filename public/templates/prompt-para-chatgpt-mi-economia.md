# Completar mi economía real para Pricing OS

Ayudame a preparar el perfil económico de **{{ACTIVIDAD}}** en **{{MONEDA}}** para importarlo en Pricing OS. No cambies la actividad ni la moneda y no mezcles esta profesión con otras.

## Paso 1 · preguntarme antes de calcular

Antes de investigar o devolver un JSON, haceme preguntas breves para obtener mis decisiones personales. No inventes ni deduzcas estos valores desde salarios o promedios de internet:

1. ingreso mensual que quiero alcanzar;
2. gastos mensuales reales de mi actividad;
3. horas facturables disponibles por mes;
4. reserva que quiero separar para impuestos, comisiones e imprevistos;
5. margen profesional deseado;
6. recargo que deseo cobrar por urgencia;
7. días de trabajo por mes;
8. semanas de descanso o vacaciones por año.

Si no conozco una respuesta, explicame qué significa y pedime que elija un supuesto. Podés sugerir alternativas de planificación claramente identificadas, pero no las presentes como datos investigados ni las decidas por mí. Aclarame que impuestos y reservas deben validarse con un profesional contable.

## Paso 2 · investigar sólo el mercado

Después de recibir mis respuestas, investigá únicamente una **tarifa freelance por hora vigente para {{ACTIVIDAD}}**. Priorizá Argentina cuando la moneda sea ARS, fuentes profesionales recientes y trabajos equivalentes. Separá tarifas freelance de salarios, costos de impresión, prendas, materiales y servicios tercerizados. Conservá URL, fecha de consulta, rango publicado y criterio de selección.

La tarifa investigada es una referencia de mercado: no reemplaza ni altera mis decisiones personales. No presentes un conjunto de valores personales inventados como un perfil sostenible válido.

## Paso 3 · devolver el archivo

Cuando estén confirmadas mis ocho decisiones y la tarifa de mercado, devolvé solamente un JSON válido, sin Markdown. Debe conservar exactamente estas nueve claves económicas, además de actividad, moneda, versión y fuentes:

```json
{
  "schemaVersion": 1,
  "actividad": "{{ACTIVIDAD}}",
  "moneda": "{{MONEDA}}",
  "tarifaManualPorHora": 0,
  "ingresoMensualObjetivo": 0,
  "gastosMensuales": 0,
  "horasFacturablesPorMes": 0,
  "reservaImpuestosPorcentaje": 0,
  "margenDeseadoPorcentaje": 0,
  "urgenciaPredeterminadaPorcentaje": 0,
  "diasDeTrabajoPorMes": 0,
  "semanasVacacionesPorAnio": 0,
  "fuentes": [
    {
      "nombre": "Nombre de la fuente",
      "url": "https://fuente-verificada.example",
      "fechaConsulta": "AAAA-MM-DD",
      "camposRelacionados": ["tarifaManualPorHora"],
      "datoUsado": "Rango o tarifa publicada",
      "notas": "Por qué es comparable con la actividad"
    }
  ]
}
```

Controlá que:

- `actividad` sea exactamente `{{ACTIVIDAD}}` y `moneda` sea exactamente `{{MONEDA}}`;
- las nueve claves económicas sean números sin comillas y no falte ninguna;
- las horas facturables sean mayores que cero;
- los importes estén en unidades completas de {{MONEDA}}, no en centavos;
- las fuentes sólo respalden la tarifa investigada; mis decisiones personales no necesitan una fuente externa;
- no haya `null`, marcadores, rangos dentro de campos numéricos ni parámetros del encargo de una cotización.
