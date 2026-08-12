# Generar el perfil manual completo de Pricing OS

Tu tarea es investigar y preparar un perfil económico inicial **completo**, listo para importar en Pricing OS.

- Actividad/profesión: **{{ACTIVIDAD}}**
- Moneda: **{{MONEDA}}**
- Mercado prioritario: **Argentina**

No cambies la actividad, no mezcles datos de otras profesiones y no uses precios internacionales como si fueran precios locales argentinos.

## Resultado obligatorio

Tenés que completar **todos** estos campos con números:

1. `tarifaManualPorHora`
2. `ingresoMensualObjetivo`
3. `gastosMensuales`
4. `horasFacturablesPorMes`
5. `reservaImpuestosPorcentaje`
6. `margenDeseadoPorcentaje`
7. `urgenciaPredeterminadaPorcentaje`
8. `diasDeTrabajoPorMes`
9. `semanasVacacionesPorAnio`

No hay un modo reducido. No entregues solamente la tarifa por hora. No uses `null`, campos vacíos, textos `REEMPLAZAR`, rangos dentro de los campos numéricos ni elimines ninguna de estas nueve claves.

## Cómo obtener cada valor

- **Tarifa manual por hora:** investigá tarifas freelance actuales de {{ACTIVIDAD}} en Argentina. Priorizá fuentes recientes, profesionales y verificables.
- **Ingreso mensual objetivo:** proponé un objetivo inicial sostenible y realista para un profesional independiente de esta actividad en Argentina. Contrastalo con ingresos locales, pero no conviertas mecánicamente un salario de empleado en tarifa freelance.
- **Gastos mensuales:** proponé una previsión inicial razonable de gastos operativos y profesionales de esta actividad. Explicá qué incluye. Es un valor editable, no una afirmación sobre los gastos reales de la persona.
- **Horas facturables por mes:** estimá horas vendibles reales, descontando administración, búsqueda de clientes, reuniones y tareas no facturables.
- **Reserva e impuestos:** proponé un porcentaje prudente para impuestos, comisiones, incobrables e imprevistos. Aclará que no constituye asesoramiento contable.
- **Margen deseado:** proponé un margen profesional sostenible y explicá el criterio.
- **Urgencia predeterminada:** proponé un recargo razonable para trabajos urgentes en esta actividad.
- **Días de trabajo por mes:** proponé un calendario mensual realista.
- **Semanas de vacaciones por año:** incorporá descanso planificado para que la tarifa sea sostenible.

Cuando un valor no exista publicado literalmente, no lo dejes vacío: generá una **propuesta inicial de planificación**, basada en supuestos explícitos y conservadores. La persona podrá modificarla después en Pricing OS.

## Investigación y trazabilidad

- Usá información actual y verificá la fecha.
- Conservá URL, fecha de consulta, dato utilizado, campo al que contribuye y criterio aplicado.
- Diferenciá claramente precios freelance, salarios de empleo y supuestos de planificación.
- Si encontrás rangos, elegí un número prudente dentro del rango para el JSON y conservá el rango original en `fuentes`.
- No inventes una fuente. Los supuestos propios deben identificarse como `supuesto de planificación`, no como datos publicados.

## Formato de respuesta

Devolvé **solamente un JSON válido**, sin Markdown ni explicaciones fuera del JSON. Sustituí todos los marcadores por números JSON sin comillas.

```json
{
  "schemaVersion": 1,
  "actividad": "{{ACTIVIDAD}}",
  "moneda": "{{MONEDA}}",
  "tarifaManualPorHora": "NUMERO_INVESTIGADO",
  "ingresoMensualObjetivo": "NUMERO_PROPUESTO",
  "gastosMensuales": "NUMERO_PROPUESTO",
  "horasFacturablesPorMes": "NUMERO_PROPUESTO",
  "reservaImpuestosPorcentaje": "NUMERO_PROPUESTO",
  "margenDeseadoPorcentaje": "NUMERO_PROPUESTO",
  "urgenciaPredeterminadaPorcentaje": "NUMERO_PROPUESTO",
  "diasDeTrabajoPorMes": "NUMERO_PROPUESTO",
  "semanasVacacionesPorAnio": "NUMERO_PROPUESTO",
  "fuentes": [
    {
      "nombre": "Nombre de la fuente o Supuesto de planificación",
      "url": "https://url-verificada.example o cadena vacía si es un supuesto",
      "fechaConsulta": "AAAA-MM-DD",
      "camposRelacionados": ["tarifaManualPorHora"],
      "datoUsado": "Dato, rango o supuesto utilizado",
      "notas": "Criterio de selección y limitaciones"
    }
  ]
}
```

## Control final obligatorio

Antes de responder verificá:

- `actividad` es exactamente `{{ACTIVIDAD}}`;
- `moneda` es exactamente `{{MONEDA}}`;
- aparecen las nueve claves económicas;
- las nueve contienen un único número mayor o igual a cero, sin comillas;
- `horasFacturablesPorMes` es mayor que cero;
- no existe ningún `null`, `REEMPLAZAR`, `NUMERO_`, campo vacío ni clave omitida;
- los importes monetarios están en unidades completas de {{MONEDA}}, no en centavos;
- cada cifra investigada o propuesta tiene trazabilidad dentro de `fuentes`;
- el resultado corresponde solamente a {{ACTIVIDAD}} en Argentina.

Si tu respuesta no cumple todas estas condiciones, corregila antes de entregarla.
