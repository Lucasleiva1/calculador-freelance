# Pricing OS

Aplicación desktop local para calcular y organizar cotizaciones freelance por servicios configurables.

## Estado

Respaldo inicial al cierre de la Fase 2:

- Tauri 2, React, Vite, TypeScript y Tailwind CSS.
- SQLite local mediante SQLx y migraciones embebidas.
- Clientes, proyectos, cotizaciones y múltiples servicios.
- Módulos de Edición de video y Programación.
- Motor configurable de reglas de pricing.
- Perfiles económicos ARS/USD y tarifa sostenible.
- Precio calculado, sugerido y final con override y desglose.
- Snapshots versionados para proteger cotizaciones históricas.
- Configuración de parámetros, reglas, presets y fuentes de mercado.
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
npm.cmd test -- --run
npm.cmd run build

cd src-tauri
cargo fmt --all -- --check
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

PDF, instalador, firma, actualizaciones automáticas, cloud, autenticación y extracción automática de mercado siguen fuera de alcance.
