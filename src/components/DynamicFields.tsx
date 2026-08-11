import type { ParameterOption, ServiceParameter } from "../domain/types";
import { applySuggestedDefaults } from "../domain/pricingEngine";
import { Field, Input, Select } from "./ui";
import { EffortInput } from "./EffortInput";
import type { EffortUnit } from "../domain/effort";

export function DynamicFields({ parameters, options, values, suggestionsEnabled, onChange }: { parameters: ServiceParameter[]; options: ParameterOption[]; values: Record<string, unknown>; suggestionsEnabled: boolean; onChange: (values: Record<string, unknown>) => void }) {
  const effective = applySuggestedDefaults(values, parameters, suggestionsEnabled);
  const set = (key: string, value: unknown) => onChange({ ...effective, [key]: value });
  return <div className="dynamic-fields editor-grid editor-grid--2">{parameters.filter((item) => item.enabled).sort((a, b) => a.sortOrder - b.sortOrder).map((parameter) => {
    const value = effective[parameter.parameterKey];
    const parameterOptions = options.filter((item) => item.parameterId === parameter.id && item.enabled);
    if (parameter.parameterKey === "estimatedHours") {
      const amount = typeof effective.effortAmount === "number" ? effective.effortAmount : null;
      const unit = ["hours", "days", "weeks"].includes(String(effective.effortUnit)) ? effective.effortUnit as EffortUnit : "hours";
      const hoursPerDay = typeof effective.hoursPerDay === "number" ? effective.hoursPerDay : 8;
      const estimatedHours = typeof value === "number" ? value : typeof value === "string" && value.trim() ? Number(value) : null;
      return <EffortInput key={parameter.id} amount={amount} unit={unit} hoursPerDay={hoursPerDay} estimatedHours={estimatedHours} onChange={(effort) => onChange({ ...effective, effortAmount: effort.amount, effortUnit: effort.unit, hoursPerDay: effort.hoursPerDay, estimatedHours: effort.estimatedHours })} />;
    }
    if (parameter.parameterType === "boolean") return <label className="dynamic-toggle" key={parameter.id}><input type="checkbox" checked={Boolean(value)} onChange={(e) => set(parameter.parameterKey, e.target.checked)} /><span><strong>{parameter.label}</strong>{parameter.description && <small>{parameter.description}</small>}</span></label>;
    if (parameter.parameterType === "single_select") return <Field key={parameter.id} label={parameter.label} hint={parameter.description ?? undefined}><Select required={parameter.required} value={String(value ?? "")} onChange={(e) => set(parameter.parameterKey, e.target.value)}><option value="">Seleccionar</option>{parameterOptions.map((option) => <option value={option.value} key={option.id}>{option.label}</option>)}</Select></Field>;
    if (parameter.parameterType === "multi_select") { const selected = Array.isArray(value) ? value as string[] : []; return <Field key={parameter.id} label={parameter.label} hint={parameter.description ?? undefined}><div className="dynamic-multi">{parameterOptions.map((option) => <label key={option.id}><input type="checkbox" checked={selected.includes(option.value)} onChange={() => set(parameter.parameterKey, selected.includes(option.value) ? selected.filter((item) => item !== option.value) : [...selected, option.value])} /> {option.label}</label>)}</div></Field>; }
    if (parameter.parameterType === "text") return <Field key={parameter.id} label={parameter.label} hint={parameter.description ?? undefined}><Input required={parameter.required} value={String(value ?? "")} onChange={(e) => set(parameter.parameterKey, e.target.value)} /></Field>;
    const isPercent = parameter.parameterType === "percentage";
    return <Field key={parameter.id} label={`${parameter.label}${isPercent ? " · %" : ""}`} hint={parameter.description ?? undefined}><Input type="number" required={parameter.required} min="0" step={parameter.parameterType === "currency" ? "0.01" : "0.01"} value={typeof value === "number" || typeof value === "string" ? value : ""} onChange={(e) => set(parameter.parameterKey, e.target.value === "" ? null : Number(e.target.value))} /></Field>;
  })}</div>;
}
