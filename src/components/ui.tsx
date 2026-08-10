import { useEffect, useId, useRef } from "react";
import type { ButtonHTMLAttributes, InputHTMLAttributes, KeyboardEvent as ReactKeyboardEvent, ReactNode, SelectHTMLAttributes } from "react";
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
  const titleId = useId();
  const modalRef = useRef<HTMLElement>(null);
  const previousFocus = useRef<HTMLElement | null>(null);

  useEffect(() => {
    previousFocus.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    const focusTimer = window.setTimeout(() => {
      modalRef.current?.querySelector<HTMLElement>("button, input, select, textarea, [tabindex]:not([tabindex='-1'])")?.focus();
    }, 0);
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    document.addEventListener("keydown", closeOnEscape);
    return () => {
      window.clearTimeout(focusTimer);
      document.removeEventListener("keydown", closeOnEscape);
      previousFocus.current?.focus();
    };
  }, [onClose]);

  function keepFocusInside(event: ReactKeyboardEvent<HTMLElement>) {
    if (event.key !== "Tab") return;
    const focusable = [...(modalRef.current?.querySelectorAll<HTMLElement>("button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex='-1'])") ?? [])];
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
    else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
  }

  return <div className="modal-backdrop" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
    <section ref={modalRef} className="modal" role="dialog" aria-modal="true" aria-labelledby={titleId} style={{ maxWidth: width }} onKeyDown={keepFocusInside}>
      <header className="modal__header"><div><span className="eyebrow">Pricing OS</span><h2 id={titleId}>{title}</h2></div><button type="button" className="icon-button" aria-label="Cerrar" onClick={onClose}><X size={20} aria-hidden="true" /></button></header>
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
