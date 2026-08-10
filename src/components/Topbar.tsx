import { ChevronDown, Moon, Plus, Settings2, Sun } from "lucide-react";
import type { Currency, ProjectSummary, Theme } from "../domain/types";
import { formatRate } from "../domain/money";

export function Topbar({
  project,
  projects,
  theme,
  usdToArsMicros,
  onProject,
  onNewProject,
  onCurrency,
  onToggleTheme,
  onSettings,
}: {
  project: ProjectSummary | null;
  projects: ProjectSummary[];
  theme: Theme;
  usdToArsMicros: number | null;
  onProject: (id: string) => void;
  onNewProject: () => void;
  onCurrency: (currency: Currency) => void;
  onToggleTheme: () => void;
  onSettings: () => void;
}) {
  return <header className="topbar">
    <div className="project-switcher">
      {project ? <label>
        <span className="sr-only">Proyecto activo</span>
        <select value={project.id} onChange={(event) => onProject(event.target.value)}>
          {projects.filter((item) => item.status === "active").map((item) => <option key={item.id} value={item.id}>{item.clientName} / {item.name}</option>)}
        </select>
        <ChevronDown size={15} aria-hidden="true" />
      </label> : <span className="project-switcher__empty">Sin proyecto activo</span>}
      <button aria-label="Nuevo proyecto" onClick={onNewProject}><Plus size={17} /></button>
    </div>
    <div className="topbar__tools">
      {project && <div className="currency-toggle" aria-label="Moneda de la cotización">
        {(["ARS", "USD"] as Currency[]).map((currency) => <button key={currency} className={project.currency === currency ? "is-active" : ""} onClick={() => onCurrency(currency)}>{currency}</button>)}
      </div>}
      <button className="rate-button" onClick={onSettings}><span>USD / ARS</span><strong>{formatRate(usdToArsMicros)}</strong><i /></button>
      <button className="theme-toggle" onClick={onToggleTheme} aria-label={theme === "warm" ? "Activar modo oscuro" : "Activar modo cálido"}>{theme === "warm" ? <Sun size={18} /> : <Moon size={18} />}<span className="theme-toggle__track"><i /></span></button>
      <button className="icon-button" onClick={onSettings} aria-label="Abrir configuración"><Settings2 size={19} /></button>
    </div>
  </header>;
}

