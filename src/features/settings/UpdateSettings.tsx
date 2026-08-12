import { useEffect, useRef, useState } from "react";
import { CheckCircle2, Download, RefreshCw, RotateCw, ShieldCheck } from "lucide-react";
import { Button } from "../../components/ui";
import { appUpdater, type AvailableAppUpdate, type UpdateProgress } from "../../services/updater";

type UpdatePhase = "idle" | "checking" | "current" | "available" | "saving" | "installing" | "error";

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

function progressLabel(progress: UpdateProgress | null) {
  if (!progress) return "Preparando descarga firmada…";
  if (progress.finished) return "Descarga completa. Instalando la nueva versión…";
  if (progress.totalBytes) return `${formatBytes(progress.downloadedBytes)} de ${formatBytes(progress.totalBytes)}`;
  return `${formatBytes(progress.downloadedBytes)} descargados`;
}

export function UpdateSettings({ onBeforeInstall }: { onBeforeInstall: () => Promise<boolean> }) {
  const [currentVersion, setCurrentVersion] = useState("…");
  const [phase, setPhase] = useState<UpdatePhase>("idle");
  const [message, setMessage] = useState("Buscá una versión nueva cuando quieras actualizar esta instalación.");
  const [progress, setProgress] = useState<UpdateProgress | null>(null);
  const [available, setAvailable] = useState<AvailableAppUpdate | null>(null);
  const updateRef = useRef<AvailableAppUpdate | null>(null);

  useEffect(() => {
    let active = true;
    void appUpdater.currentVersion().then((version) => { if (active) setCurrentVersion(version); }).catch(() => { if (active) setCurrentVersion("desconocida"); });
    return () => {
      active = false;
      const update = updateRef.current;
      updateRef.current = null;
      if (update) void update.dispose();
    };
  }, []);

  async function checkNow() {
    if (["checking", "saving", "installing"].includes(phase)) return;
    const previous = updateRef.current;
    updateRef.current = null;
    setAvailable(null);
    if (previous) await previous.dispose();
    setPhase("checking");
    setMessage("Consultando el canal estable de GitHub Releases…");
    setProgress(null);
    try {
      const update = await appUpdater.check();
      if (!update) {
        setPhase("current");
        setMessage("Ya tenés la versión estable más reciente.");
        return;
      }
      updateRef.current = update;
      setAvailable(update);
      setPhase("available");
      setMessage(`La versión ${update.version} está lista para descargar e instalar.`);
    } catch (error) {
      setPhase("error");
      setMessage(`No se pudo consultar GitHub Releases: ${String(error)}`);
    }
  }

  async function install() {
    const update = updateRef.current;
    if (!update || phase !== "available") return;
    setPhase("saving");
    setMessage("Guardando los cambios pendientes antes de actualizar…");
    try {
      if (!(await onBeforeInstall())) {
        setPhase("error");
        setMessage("No se pudo guardar todo. Corregí el error de guardado antes de instalar la actualización.");
        return;
      }
      setPhase("installing");
      setMessage("Descargando la actualización firmada…");
      await update.install((next) => {
        setProgress(next);
        setMessage(progressLabel(next));
      });
    } catch (error) {
      setPhase("error");
      setMessage(`No se pudo instalar la actualización: ${String(error)}`);
    }
  }

  const update = available;
  const percent = progress?.totalBytes ? Math.min(100, Math.round(progress.downloadedBytes / progress.totalBytes * 100)) : null;
  return <div className="settings-form update-settings">
    <section>
      <span className="eyebrow">GitHub Releases · canal estable</span>
      <h2>Actualizaciones de Pricing OS</h2>
      <p>Una sola instalación. La actualización reemplaza los archivos de la versión anterior y vuelve a abrir Pricing OS; tus proyectos, economía y perfiles locales se conservan.</p>
      <div className="update-version-card">
        <div><span>Versión instalada</span><strong>v{currentVersion}</strong></div>
        <div><span>Versión disponible</span><strong>{update ? `v${update.version}` : "—"}</strong></div>
        <div className="update-security"><ShieldCheck size={20} /><span><strong>Firma obligatoria</strong><small>Tauri verifica el asset antes de instalarlo.</small></span></div>
      </div>
      <div className={`update-status update-status--${phase}`} role={phase === "error" ? "alert" : "status"} aria-live="polite">
        {phase === "current" ? <CheckCircle2 size={20} /> : phase === "installing" ? <Download size={20} /> : <RotateCw className={["checking", "saving"].includes(phase) ? "is-spinning" : ""} size={20} />}
        <div><strong>{phase === "error" ? "La comprobación no pudo completarse" : phase === "available" ? "Actualización disponible" : phase === "installing" ? "Actualizando Pricing OS" : "Estado de actualización"}</strong><span>{message}</span></div>
      </div>
      {phase === "installing" && <div className="update-progress" aria-label={percent == null ? "Descargando actualización" : `Descarga ${percent}%`}><span style={{ width: percent == null ? "12%" : `${percent}%` }} /></div>}
      {update && <div className="update-release"><strong>Novedades de v{update.version}</strong><p>{update.notes || "Esta versión no incluye notas adicionales."}</p>{update.date && <small>Publicada: {new Date(update.date).toLocaleString("es-AR")}</small>}</div>}
      <div className="update-actions">
        <Button type="button" disabled={["checking", "saving", "installing"].includes(phase)} onClick={() => void checkNow()}><RefreshCw size={16} /> {phase === "checking" ? "Buscando…" : "Buscar actualizaciones"}</Button>
        {phase === "available" && <Button type="button" variant="accent" onClick={() => void install()}><Download size={16} /> Descargar e instalar</Button>}
      </div>
    </section>
  </div>;
}
