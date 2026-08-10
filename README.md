# Pricing OS

Aplicación desktop local para calcular y organizar cotizaciones freelance por servicios configurables.

## Estado

Fase 3 implementada sobre el respaldo de Fase 2:

- Tauri 2, React, Vite, TypeScript y Tailwind CSS.
- SQLite local mediante SQLx y migraciones embebidas.
- Clientes, proyectos, cotizaciones y múltiples servicios.
- Módulos de Edición de video y Programación.
- Motor configurable de reglas de pricing.
- Perfiles económicos ARS/USD y tarifa sostenible.
- Precio calculado, sugerido y final con override y desglose.
- Snapshots versionados para proteger cotizaciones históricas.
- Configuración de parámetros, reglas, presets y fuentes de mercado.
- Source Registry dinámico: alta, edición, archivo, restauración, prueba y observaciones manuales.
- Market Intelligence Engine en Rust con adquisición HTTP conservadora, adapters, normalización, validación, caché, cooldown y logs.
- Adapters específicos para BCRA, Tarifario.org, YunoJuno y RemoteJobs.lat; las fuentes no verificadas permanecen manuales.
- Observaciones deduplicadas y snapshots de mercado inmutables con mediana, percentiles, comparabilidad y conversiones auditables.
- Mercado global, referencia por servicio, fuentes utilizadas y explicación determinística de la sugerencia.
- Warm/Dark Mode y layout responsive desde 820 × 620.

## Desarrollo

```powershell
npm.cmd install
npm.cmd run tauri dev
```

## Verificación

```powershell
npm.cmd run lint
npm.cmd run typecheck
npm.cmd run test
npm.cmd run build

cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Build Windows sin instalador

```powershell
npm.cmd run tauri build -- --no-bundle
```

El ejecutable resultante se genera en `src-tauri/target/release/pricing-os.exe` y no se versiona.

## Persistencia

La base de datos se guarda en el directorio de datos de la aplicación, fuera del repositorio. El esquema se reconstruye mediante las migraciones de `src-tauri/migrations`.

PDF, instalador, firma, actualizaciones automáticas, cloud, autenticación y automatización browser con Playwright siguen fuera de alcance. `AUTO_BROWSER` está aislado y no ejecuta una fuente hasta incorporar un sidecar explícitamente aprobado.

## Mercado y seguridad

La investigación sólo comienza por acción del usuario. Las consultas salen desde Rust, aceptan exclusivamente HTTPS público, validan redirecciones y resolución DNS, bloquean destinos locales/privados y limitan cada respuesta a 1 MB. CAPTCHA, login, paywall, 401/403, 429 y challenges detienen la fuente; no existe evasión anti-bot.

Una fuente personalizada siempre nace `MANUAL + UNREVIEWED`. Los datos salariales, metodológicos o marcados como contexto se conservan, pero no participan en la sugerencia freelance. El precio final nunca es modificado por Mercado.
