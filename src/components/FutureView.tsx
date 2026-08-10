import type { LucideIcon } from "lucide-react";
import { Construction } from "lucide-react";
import { EmptyState } from "./ui";

export function FutureView({ eyebrow, title, description, icon: Icon = Construction }: { eyebrow: string; title: string; description: string; icon?: LucideIcon }) {
  return <div className="view-page future-view"><EmptyState eyebrow={eyebrow} title={title} description={description} action={<div className="future-stamp"><Icon size={19} /> Preparado para Fase 2</div>} /></div>;
}

