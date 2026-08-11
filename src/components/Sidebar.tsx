import {
  Archive,
  BriefcaseBusiness,
  FileClock,
  FolderOpen,
  Layers3,
  PlusSquare,
  Settings,
  TrendingUp,
  Users,
} from "lucide-react";
import { APP_NAME, APP_VERSION } from "../app/brand";

export type AppSection = "workspace" | "projects" | "clients" | "services" | "market" | "history" | "settings";

const items: Array<{ id: AppSection; label: string; icon: typeof FolderOpen }> = [
  { id: "workspace", label: "Cotización", icon: BriefcaseBusiness },
  { id: "projects", label: "Proyectos", icon: FolderOpen },
  { id: "clients", label: "Clientes", icon: Users },
  { id: "services", label: "Servicios", icon: Layers3 },
  { id: "market", label: "Mercado", icon: TrendingUp },
  { id: "history", label: "Cotizaciones", icon: FileClock },
];

export function Sidebar({ section, onSection, onNewProject }: { section: AppSection; onSection: (section: AppSection) => void; onNewProject: () => void }) {
  return <aside className="sidebar">
    <div className="brand"><span className="brand__mark" aria-hidden="true"><i /><i /></span><span>{APP_NAME}</span></div>
    <nav className="nav" aria-label="Navegación principal">
      <button className="nav__new" onClick={onNewProject}><PlusSquare size={20} /><span>Nuevo proyecto</span></button>
      {items.map(({ id, label, icon: Icon }) => <button key={id} className={`nav__item ${section === id ? "is-active" : ""}`} onClick={() => onSection(id)}><Icon size={20} /><span>{label}</span></button>)}
    </nav>
    <div className="sidebar__footer">
      <button className={`nav__item ${section === "settings" ? "is-active" : ""}`} onClick={() => onSection("settings")}><Settings size={20} /><span>Configuración</span></button>
      <div className="build-label"><Archive size={12} /> v{APP_VERSION} · local</div>
    </div>
  </aside>;
}
