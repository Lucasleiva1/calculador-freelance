import { Archive, ArrowUpRight, Plus, RotateCcw, Search } from "lucide-react";
import { useMemo, useState } from "react";
import type { ProjectSummary } from "../../domain/types";
import { formatMoney } from "../../domain/money";
import { Button, EmptyState } from "../../components/ui";

export function ProjectsView({ projects, onNew, onOpen, onArchive }: {
  projects: ProjectSummary[];
  onNew: () => void;
  onOpen: (id: string) => void;
  onArchive: (project: ProjectSummary, archived: boolean) => Promise<void>;
}) {
  const [search, setSearch] = useState("");
  const [showArchived, setShowArchived] = useState(false);
  const filtered = useMemo(() => projects.filter((project) =>
    (showArchived || project.status === "active")
    && `${project.name} ${project.clientName}`.toLowerCase().includes(search.toLowerCase())),
  [projects, search, showArchived]);

  return <div className="view-page">
    <header className="page-header"><div><span className="eyebrow">Trabajo con autosave</span><h1>Proyectos</h1><p>Acá recuperás el borrador vivo de cada trabajo. Los campos, módulos y precios actuales se guardan automáticamente; los cortes inmutables quedan en Cotizaciones.</p></div><Button variant="accent" onClick={onNew}><Plus size={17} /> Nuevo proyecto</Button></header>
    <div className="list-toolbar"><label className="search-box"><Search size={17} /><input aria-label="Buscar proyectos" value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Buscar proyecto o cliente" /></label><label className="check-label"><input type="checkbox" checked={showArchived} onChange={(event) => setShowArchived(event.target.checked)} /> Mostrar archivados</label></div>
    {filtered.length === 0 ? <EmptyState title="No hay proyectos todavía" description="Creá el primero para abrir una cotización draft con guardado automático." action={<Button onClick={onNew}>Nuevo proyecto</Button>} /> : <div className="editorial-table"><div className="editorial-table__head project-columns"><span>Proyecto</span><span>Cliente</span><span>Moneda</span><span>Total actual</span><span>Actualizado</span><span /></div>{filtered.map((project) => <div className="editorial-table__row project-columns" key={project.id}><div><strong>{project.name}</strong><small className={`state-label state-label--${project.status}`}>{project.status === "active" ? "Activo" : "Archivado"}</small></div><span>{project.clientName}</span><span>{project.currency}</span><div><strong>{formatMoney(project.totalMinor, project.currency)}</strong>{project.unpricedCount > 0 && <small>{project.unpricedCount} pendiente{project.unpricedCount === 1 ? "" : "s"}</small>}</div><span>{new Intl.DateTimeFormat("es-AR", { dateStyle: "medium" }).format(new Date(project.updatedAt))}</span><div className="row-actions"><button title="Abrir proyecto" onClick={() => onOpen(project.id)}><ArrowUpRight size={16} /></button><button title={project.status === "active" ? "Archivar" : "Restaurar"} onClick={() => onArchive(project, project.status === "active")}>{project.status === "active" ? <Archive size={16} /> : <RotateCcw size={16} />}</button></div></div>)}</div>}
  </div>;
}
