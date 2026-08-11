# Datos para importar en Mi economía de Pricing OS

Buscá y/o estimá únicamente los datos que se puedan fundamentar. No inventes cifras: si no podés determinar un valor, devolvelo como `null` y explicalo dentro de `fuentes`.

## Contexto que debo completar antes de enviarte este archivo

- País/mercado: `[ej. Argentina]`
- Moneda: `[ARS o USD]`
- Actividad: `[ej. edición de video freelance]`
- Nivel y tipo de clientes: `[ej. intermedio, pymes y agencias]`
- Objetivo mensual personal: `[opcional]`
- Gastos mensuales personales/profesionales: `[opcional]`
- Horas que quiero facturar por mes: `[opcional]`

## Qué necesito que investigues

1. Una tarifa horaria freelance de referencia para esta actividad, mercado y nivel.
2. La fecha de consulta y enlaces concretos de las fuentes utilizadas.
3. Si hay varios valores, elegí uno conservador y explicá brevemente el criterio en `fuentes`.
4. No confundas salario mensual de empleo con tarifa freelance por hora.
5. No reemplaces mis valores personales de ingresos, gastos u horas si no te los di: dejalos en `null`.

## Respuesta obligatoria

Devolvé **solamente** un bloque JSON válido, sin texto antes ni después. Todos los importes monetarios deben estar expresados en unidades completas de la moneda, no en centavos. Usá punto decimal sólo si hace falta.

```json
{
  "schemaVersion": 1,
  "moneda": "ARS",
  "tarifaManualPorHora": null,
  "ingresoMensualObjetivo": null,
  "gastosMensuales": null,
  "horasFacturablesPorMes": null,
  "reservaImpuestosPorcentaje": null,
  "margenDeseadoPorcentaje": null,
  "urgenciaPredeterminadaPorcentaje": null,
  "diasDeTrabajoPorMes": null,
  "semanasVacacionesPorAnio": null,
  "fuentes": [
    {
      "nombre": "",
      "url": "",
      "fechaConsulta": "AAAA-MM-DD",
      "datoUsado": "",
      "notas": ""
    }
  ]
}
```

## Campos que entiende Pricing OS

| Campo | Qué significa | Obligatorio |
| --- | --- | --- |
| `moneda` | `ARS` o `USD` | Sí |
| `tarifaManualPorHora` | Tarifa elegida por hora. Si existe, tiene prioridad. | No, pero alcanza para cotizar |
| `ingresoMensualObjetivo` | Ingreso bruto mensual deseado | No |
| `gastosMensuales` | Gastos mensuales personales y profesionales | No |
| `horasFacturablesPorMes` | Horas que realmente se pueden vender al mes | No |
| `reservaImpuestosPorcentaje` | Reserva para impuestos e imprevistos | No |
| `margenDeseadoPorcentaje` | Margen deseado sobre el costo sostenible | No |
| `urgenciaPredeterminadaPorcentaje` | Recargo de urgencia por defecto | No |
| `diasDeTrabajoPorMes` | Días de trabajo por mes | No |
| `semanasVacacionesPorAnio` | Semanas de vacaciones anuales | No |

La aplicación acepta este JSON directamente. También acepta un TXT, MD o PDF de texto que contenga este mismo JSON o líneas del tipo `campo: valor`.
