# Contrato de sincronización de parámetros de Pricing OS

## Objetivo obligatorio

Pricing OS no puede pedir, investigar, guardar, calcular o mostrar información distinta según la pantalla o el archivo utilizado. Cuando se agregue, elimine, renombre o cambie un parámetro de una profesión, hay que verificar todas sus representaciones y actualizar juntas las que correspondan.

Esta revisión es obligatoria para Edición de video, Programación, el futuro motor de Diseño gráfico/estampas y cualquier profesión o motor nuevo.

No se debe copiar un dato entre profesiones por comodidad. Cada profesión conserva su economía manual, parámetros, fuentes y contexto de cálculo propios.

## Regla de pertinencia

No todos los parámetros deben aparecer en todos los lugares. Antes de implementar un cambio hay que clasificarlo:

1. **Economía personal/profesional:** define la tarifa local sostenible de la persona, por ejemplo ingreso objetivo, gastos, horas facturables, impuestos o tarifa manual. Debe sincronizarse con Mi economía, su almacenamiento, la guía para IA, la plantilla JSON y el importador.
2. **Característica del trabajo:** describe el encargo concreto, por ejemplo cantidad de videos para subir, cantidad de pantallas, piezas, revisiones, minutos, días o complejidad. Debe sincronizarse con el formulario de cotización, validación, persistencia, cálculo, desglose, resumen y presupuesto/PDF.
3. **Dato que modifica la investigación de mercado:** permite buscar comparables correctos, por ejemplo especialidad, tipo de proyecto, nivel, unidad, duración o cantidad. Debe incorporarse al contexto de investigación, filtros de fuentes, normalización, comparación y explicación del resultado automático.
4. **Dato informativo:** se muestra o conserva, pero no modifica el precio ni la búsqueda. Debe quedar identificado como informativo y no puede influir accidentalmente en el cálculo.

Un parámetro puede pertenecer a más de una categoría. Si no corresponde a una superficie, no se agrega allí, pero la decisión debe ser consciente y comprobable.

## Matriz de impacto obligatoria

Ante cualquier cambio de parámetro, revisar como mínimo:

| Superficie | Pregunta obligatoria |
| --- | --- |
| Definición del motor | ¿Pertenece a qué profesión y tiene una clave estable? |
| Formulario | ¿La persona puede cargarlo con una etiqueta, unidad, ayuda y valores sensatos? |
| Valores predeterminados/presets | ¿Necesita un valor inicial o debe quedar pendiente? |
| Validación | ¿Qué valores son válidos, obligatorios o incompatibles? |
| Persistencia y migración | ¿Se guarda y se recupera sin mezclarse con otra profesión? |
| Cálculo | ¿Modifica horas, costos, multiplicadores, recargos o sólo informa? |
| Desglose y explicación | ¿El usuario puede ver cómo afectó el importe? |
| Fuentes automáticas | ¿Se necesita para buscar o filtrar comparables de esa especialidad? |
| Guía para IA | ¿La IA necesita preguntarlo o investigarlo para producir datos utilizables? |
| Plantilla/importador | ¿Debe existir una clave importable, sin `null` silenciosos? |
| Resumen, historial y snapshot | ¿Debe quedar registrado para reproducir el precio posteriormente? |
| Presupuesto/PDF | ¿Es un dato que el cliente necesita ver? |
| Pruebas | ¿Hay cobertura de profesión correcta, cálculo y no contaminación entre motores? |

## Regla especial para documentos destinados a una IA

- La descarga debe indicar automáticamente la profesión y moneda seleccionadas.
- Nunca debe presentar una lista de campos en `null` como una respuesta aceptable.
- Si faltan decisiones personales, la IA debe preguntarlas antes de generar un perfil completo; no puede inventarlas usando promedios de internet.
- Si se solicita solamente una tarifa manual rápida, los campos no utilizados se omiten en lugar de escribirse como `null`.
- Los datos investigables deben exigir fuentes actuales, fecha, URL, unidad, región y criterio de selección.
- Un archivo que declara otra profesión debe rechazarse antes de aplicarse.
- La guía y la plantilla deben evolucionar junto con los campos que el importador entiende.
- Un documento de economía no debe fingir que los parámetros de un encargo son datos personales. Si la IA necesita ambos tipos de datos, el documento debe separar claramente **economía de la profesión** y **características del trabajo**.

## Ejemplo obligatorio: cantidad de videos en Programación

Si Programación incorpora un servicio donde se cobra por recibir, preparar o subir videos, `cantidadDeVideos` puede afectar el trabajo y posiblemente la investigación de mercado.

Antes de darlo por terminado hay que decidir y verificar:

- dónde se carga la cantidad y qué unidad utiliza;
- si suma tiempo por video, un costo por unidad o ambos;
- si requiere datos relacionados, como duración, peso, plataforma o preparación;
- cómo aparece en el desglose y en el presupuesto del cliente;
- si las fuentes automáticas deben buscar precios por carga, pieza, minuto, hora o proyecto;
- si el documento para IA debe preguntar la cantidad para obtener una referencia comparable;
- cómo queda guardado en el snapshot para reproducir el presupuesto;
- que el cambio no altere Edición de video ni su economía manual salvo que exista una decisión expresa equivalente para ese motor.

## Definición de terminado

Un cambio de parámetro sólo está terminado cuando:

1. fue clasificado mediante la regla de pertinencia;
2. se revisó toda la matriz de impacto;
3. se actualizaron juntas todas las superficies pertinentes;
4. se comprobó que no hereda ni contamina datos de otra profesión;
5. las descargas para IA y sus importadores siguen alineados cuando el dato les corresponde;
6. existen pruebas que fallarían si esa alineación se rompe;
7. la aplicación nativa fue validada siguiendo `RUNBOOK_PRICING_OS.md`.

Si una superficie pertinente queda pendiente, hay que informarlo claramente y el cambio no puede presentarse como completo.
