import { useEffect, useRef } from "react";
import { Circle, CircleAlert, CircleCheck, LoaderCircle, XCircle } from "lucide-react";
import { Button } from "./ui";

export type PriceProgressPhase = "local" | "market" | "international" | "complete" | "error";

export interface PriceCalculationProgressState {
  mode: "calculate" | "refresh";
  phase: PriceProgressPhase;
  jobId: string | null;
  localReady: boolean | null;
  completedSources: number;
  totalSources: number;
  errorStep?: "local" | "market" | "international";
  error?: string;
}

export function PriceCalculationProgress({
  state,
  onCancel,
  onDismiss,
}: {
  state: PriceCalculationProgressState;
  onCancel: () => void;
  onDismiss: () => void;
}) {
  const dialogRef = useRef<HTMLElement>(null);
  useEffect(() => { dialogRef.current?.focus(); }, []);

  const sourceRatio = state.totalSources > 0 ? state.completedSources / state.totalSources : 0;
  const progress = state.phase === "local" ? 14
    : state.phase === "market" ? 34 + Math.round(sourceRatio * 42)
      : state.phase === "international" ? 88
        : state.phase === "complete" ? 100
          : state.errorStep === "local" ? 14 : state.errorStep === "international" ? 88 : Math.max(34, 34 + Math.round(sourceRatio * 42));
  const title = state.mode === "calculate" ? "Calculando tus 3 precios" : "Actualizando tus 3 precios";
  const isError = state.phase === "error";
  const isRunning = !isError && state.phase !== "complete";

  const localStatus = state.phase === "local" ? "active"
    : state.localReady === false ? "warning"
      : state.errorStep === "local" ? "error" : "done";
  const marketStatus = state.phase === "local" ? "pending"
    : state.phase === "market" ? "active"
      : state.errorStep === "market" ? "error"
        : ["international", "complete"].includes(state.phase) ? "done" : "pending";
  const internationalStatus = state.phase === "international" ? "active"
    : state.phase === "complete" ? "done"
      : state.errorStep === "international" ? "error" : "pending";

  return <div className="price-progress-backdrop">
    <section ref={dialogRef} className="price-progress" role="dialog" aria-modal="true" aria-labelledby="price-progress-title" tabIndex={-1}>
      <header>
        <span className="eyebrow">Pricing OS · cálculo asistido</span>
        <h2 id="price-progress-title">{isError ? "No pudimos terminar" : state.phase === "complete" ? "Tus precios están listos" : title}</h2>
        <p>{isError ? state.error : state.phase === "complete" ? "Ya podés comparar las tres referencias y elegir la adecuada para este cliente." : "Estamos procesando cada referencia por separado."}</p>
      </header>
      <div className="price-progress__track" role="progressbar" aria-label="Progreso del cálculo" aria-valuemin={0} aria-valuemax={100} aria-valuenow={progress}><i style={{ width: `${progress}%` }} /></div>
      <ol className="price-progress__steps">
        <ProgressStep status={localStatus} label="Precio local / sostenible" detail={localDetail(state)} />
        <ProgressStep status={marketStatus} label="Precio de mercado Argentina" detail={marketDetail(state)} />
        <ProgressStep status={internationalStatus} label="Precio internacional" detail={internationalDetail(state)} />
      </ol>
      <footer>
        {isRunning && state.jobId && <Button type="button" variant="ghost" onClick={onCancel}>Cancelar actualización</Button>}
        {isError && <Button type="button" variant="accent" onClick={onDismiss}>Cerrar</Button>}
        {state.phase === "complete" && <span><CircleCheck size={15} /> Cálculo finalizado</span>}
      </footer>
    </section>
  </div>;
}

type StepStatus = "pending" | "active" | "done" | "warning" | "error";

function ProgressStep({ status, label, detail }: { status: StepStatus; label: string; detail: string }) {
  const Icon = status === "active" ? LoaderCircle : status === "done" ? CircleCheck : status === "warning" || status === "error" ? CircleAlert : Circle;
  return <li className={`price-progress__step price-progress__step--${status}`}>
    <Icon className={status === "active" ? "spin" : undefined} size={21} aria-hidden="true" />
    <div><strong>{label}</strong><span>{detail}</span></div>
    {status === "done" && <small>Listo</small>}
    {status === "warning" && <small>Pendiente</small>}
    {status === "error" && <XCircle size={15} aria-hidden="true" />}
  </li>;
}

function localDetail(state: PriceCalculationProgressState) {
  if (state.phase === "local") return "Calculando con los parámetros de tu economía manual…";
  if (state.localReady === false) return "Faltan datos manuales; los precios automáticos continúan igualmente.";
  return state.mode === "refresh" ? "Se conserva el cálculo sostenible de tus datos." : "Calculado exclusivamente con tus parámetros manuales.";
}

function marketDetail(state: PriceCalculationProgressState) {
  if (state.phase === "market") return state.totalSources > 0
    ? `Verificando referencias locales · ${state.completedSources} de ${state.totalSources}`
    : "Preparando las fuentes profesionales argentinas…";
  if (["international", "complete"].includes(state.phase)) return "Referencia comercial argentina actualizada.";
  if (state.errorStep === "market") return "No se pudo completar la consulta de mercado.";
  return "Esperando el cálculo local.";
}

function internationalDetail(state: PriceCalculationProgressState) {
  if (state.phase === "international") return "Convirtiendo y separando las referencias globales…";
  if (state.phase === "complete") return "Referencia global lista en ARS y USD.";
  if (state.errorStep === "international") return "No se pudo completar la referencia internacional.";
  return "Se calculará después del precio de mercado.";
}
