# Ejecutar y verificar Pricing OS (aplicación local de Windows)

Leé este archivo antes de abrir, compilar o verificar Pricing OS en una nueva sesión.

## Regla principal

Pricing OS es una aplicación de escritorio Tauri. La forma normal de usarla es una **ventana nativa de Windows**, no un servidor web y no una página en el navegador.

No ejecutes `npm run dev` ni abras `http://localhost:1420` para entregarle la aplicación al usuario, salvo que éste pida expresamente desarrollo web. La aplicación terminada debe abrir el ejecutable de producción y debe funcionar aunque no haya ningún servidor local en ejecución.

## Procedimiento correcto

1. Desde la raíz del proyecto, validá los cambios relevantes:

   ```powershell
   npm.cmd run typecheck
   npm.cmd run lint
   npm.cmd test -- --run
   cargo test --manifest-path src-tauri\Cargo.toml --lib --offline
   ```

2. Si existe una instancia de `pricing-os.exe` ejecutándose desde `src-tauri\target\release\pricing-os.exe`, no la mates a la fuerza: puede tener datos sin guardar. Pedí al usuario que cierre la ventana o realizá un cierre normal que confirme que el proceso desapareció.

3. Generá el ejecutable con el comando oficial de Tauri:

   ```powershell
   npm.cmd run tauri -- build
   ```

   Este comando ejecuta también el build de Vite e incorpora `dist` dentro de la aplicación. El ejecutable correcto queda en:

   ```text
   src-tauri\target\release\pricing-os.exe
   ```

4. Abrí **ese** ejecutable como ventana de Windows. No inicies Vite ni navegues a localhost.

## Error que ocurrió el 2026-08-11 y su causa

Se intentó compilar con `cargo build --release` directamente hacia otro directorio y después se abrió ese binario. Fue incorrecto como forma de lanzar la app: no recibió la configuración de producción que inyecta la CLI de Tauri y la WebView intentó resolver la URL de desarrollo `http://localhost:1420`. El resultado fue `ERR_CONNECTION_REFUSED`.

La solución correcta fue:

- cerrar ese proceso de prueba, que no había cargado la interfaz ni editado cotizaciones;
- construir con `npm.cmd run tauri -- build`;
- abrir únicamente `src-tauri\target\release\pricing-os.exe`.

Además, `src-tauri/src/main.rs` debe conservar esta línea para que el binario de producción sea gráfico y no abra una consola técnica:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

## Cómo comprobar que realmente está abierta y funciona

No afirmes que está abierta sólo porque existe un proceso. Confirmá todos estos puntos:

1. El proceso `pricing-os.exe` responde y su ruta corresponde al ejecutable de producción.
2. Existe una ventana nativa cuyo título sea **Pricing OS**, visible, no minimizada y de tamaño real (más de 300 × 200 píxeles).
3. No hay una consola técnica como única ventana. En una versión correcta hay una ventana real de Pricing OS; las ventanas pequeñas sin título del sistema no cuentan.
4. Una comprobación visual muestra la interfaz de cotizaciones, no una pantalla `localhost` ni `ERR_CONNECTION_REFUSED`.
5. La interfaz carga los datos guardados y permite ver los controles reales. Por ejemplo, el estado guardado y los botones **Calcular estimado** / **Configurar tarifa** deben aparecer cuando la cotización aún no tiene tarifa ARS.

Si falla cualquiera de esos puntos, informalo como fallo; no digas que la aplicación está abierta o funcionando.

## Nota sobre el precio vacío

Un total `—` no implica que la ventana esté rota. En la cotización de prueba del 2026-08-11 la aplicación mostraba correctamente el requisito pendiente: configurar la economía o tarifa en ARS. Una vez guardada una tarifa manual por hora o un perfil económico completo en la misma moneda, **Calcular estimado** debe generar el importe.

## Cambios de la importación de economía

En **Configuración → Mi economía** la persona puede seguir cargando los campos manualmente o usar **Importar archivo**. La importación es local y admite JSON, TXT, Markdown y PDF con texto seleccionable. Primero muestra una revisión, luego la persona debe pulsar **Aplicar al formulario** y finalmente **Guardar economía**; no guarda automáticamente.

En la misma pantalla se pueden descargar todas las veces que hagan falta:

- `Plantilla JSON`
- `Guía para IA`

Los archivos empaquetados están en `public/templates/` y se incluyen en la compilación final bajo `dist/templates/`.

## Actualizaciones firmadas por GitHub Releases

Pricing OS usa el updater oficial de Tauri v2 contra este endpoint estable:

```text
https://github.com/Lucasleiva1/calculador-freelance/releases/latest/download/latest.json
```

La clave pública está embebida en `src-tauri/tauri.conf.json`. La clave privada y su contraseña se guardan fuera del repositorio, en `%APPDATA%\Pricing OS\updater\`; nunca deben imprimirse, copiarse al workspace, subirse a Git ni adjuntarse a una Release. No regeneres la clave si esos archivos existen: las instalaciones anteriores sólo aceptarán assets firmados por su par original.

Cada Release para Windows debe publicar exactamente el instalador NSIS `.exe`, su `.exe.sig` y `latest.json`. El manifiesto debe incluir `windows-x86_64-nsis` y `windows-x86_64`, apuntando al mismo asset firmado. La primera instalación del NSIS con este updater es manual; desde esa versión en adelante, **Configuración → Actualizaciones** reemplaza la instalación existente y reinicia la aplicación conservando los datos de `%APPDATA%`.

Antes de construir assets firmados, seguí completamente la skill global `tauri-github-release-updater`, alineá la versión en los cinco archivos indicados por esa guía y cargá la clave sólo mediante variables de entorno de la sesión de build.
