import type { ButtonHTMLAttributes, InputHTMLAttributes, ReactNode, SelectHTMLAttributes } from "react";
import { X } from "lucide-react";

export function Button({ className = "", variant = "default", ...props }: ButtonHTMLAttributes<HTMLButtonElement> & { variant?: "default" | "accent" | "ghost" | "danger" }) {
  return <button className={`button button--${variant} ${className}`} {...props} />;
}

export function Field({ label, hint, children, className = "" }: { label: string; hint?: string; children: ReactNode; className?: string }) {
  return <label className={`field ${className}`}><span className="field__label">{label}</span>{children}{hint && <span className="field__hint">{hint}</span>}</label>;
}

export function Input(props: InputHTMLAttributes<HTMLInputElement>) {
  return <input className={`input ${props.className ?? ""}`} {...props} />;
}

export function Select(props: SelectHTMLAttributes<HTMLSelectElement>) {
  return <select className={`input select ${props.className ?? ""}`} {...props} />;
}

export function Modal({ title, onClose, children, width = "620px" }: { title: string; onClose: () => void; children: ReactNode; width?: string }) {
  return <div className="modal-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
    <section className="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title" style={{ maxWidth: width }}>
      <header className="modal__header"><div><span className="eyebrow">Pricing OS</span><h2 id="modal-title">{title}</h2></div><button className="icon-button" aria-label="Cerrar" onClick={onClose}><X size={20} /></button></header>
      {children}
    </section>
  </div>;
}

export function EmptyState({ eyebrow, title, description, action }: { eyebrow?: string; title: string; description: string; action?: ReactNode }) {
  return <div className="empty-state">{eyebrow && <span className="eyebrow">{eyebrow}</span>}<h2>{title}</h2><p>{description}</p>{action && <div className="empty-state__action">{action}</div>}</div>;
}

export function StatusDot({ tone = "accent" }: { tone?: "accent" | "muted" | "danger" }) {
  return <span className={`status-dot status-dot--${tone}`} aria-hidden="true" />;
}

