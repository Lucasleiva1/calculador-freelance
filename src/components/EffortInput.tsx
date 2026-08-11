import type { EffortUnit } from "../domain/effort";
import { DEFAULT_HOURS_PER_DAY, effortToHours, effortUnitLabel, hoursToEffort } from "../domain/effort";
import { Field, Input, Select } from "./ui";

export interface EffortValue {
  amount: number | null;
  unit: EffortUnit;
  hoursPerDay: number;
  estimatedHours: number | null;
}

export function EffortInput({
  amount,
  unit,
  hoursPerDay,
  estimatedHours,
  onChange,
}: {
  amount: number | null;
  unit: EffortUnit;
  hoursPerDay: number;
  estimatedHours: number | null;
  onChange: (value: EffortValue) => void;
}) {
  const safeUnit: EffortUnit = ["hours", "days", "weeks"].includes(unit) ? unit : "hours";
  const safeHoursPerDay = Number.isFinite(hoursPerDay) && hoursPerDay > 0 ? hoursPerDay : DEFAULT_HOURS_PER_DAY;
  const shownAmount = amount ?? hoursToEffort(estimatedHours, safeUnit, safeHoursPerDay);
  const convertedHours = effortToHours(shownAmount, safeUnit, safeHoursPerDay);
  const shortcuts = safeUnit === "hours" ? [4, 8, 12, 16, 24] : safeUnit === "days" ? [1, 2, 3, 5, 7] : [1, 2, 3, 4];

  function emit(nextAmount: number | null, nextUnit = safeUnit, nextHoursPerDay = safeHoursPerDay) {
    onChange({
      amount: nextAmount,
      unit: nextUnit,
      hoursPerDay: nextHoursPerDay,
      estimatedHours: effortToHours(nextAmount, nextUnit, nextHoursPerDay),
    });
  }

  function changeUnit(nextUnit: EffortUnit) {
    const nextAmount = hoursToEffort(convertedHours, nextUnit, safeHoursPerDay);
    emit(nextAmount, nextUnit, safeHoursPerDay);
  }

  return <div className="effort-input">
    <div className="editor-grid editor-grid--2">
      <Field label="Tiempo estimado" hint="Podés cargar horas, días o semanas.">
        <div className="with-shortcuts">
          <Input aria-label="Tiempo estimado" type="number" min="0" step="0.25" value={shownAmount ?? ""} onChange={(event) => emit(event.target.value === "" ? null : Math.max(0, Number(event.target.value)))} />
          <div>{shortcuts.map((value) => <button type="button" key={value} onClick={() => emit(value)}>{value}</button>)}</div>
        </div>
      </Field>
      <Field label="Unidad de tiempo">
        <Select aria-label="Unidad de tiempo" value={safeUnit} onChange={(event) => changeUnit(event.target.value as EffortUnit)}>
          <option value="hours">Horas</option>
          <option value="days">Días</option>
          <option value="weeks">Semanas · 7 días</option>
        </Select>
      </Field>
    </div>
    {safeUnit !== "hours" && <div className="effort-day-length">
      <Field label="Horas de trabajo por día" hint="Elegí tu jornada real; 8 h es el valor inicial.">
        <div className="with-shortcuts">
          <Input aria-label="Horas de trabajo por día" type="number" min="1" max="24" step="0.5" value={safeHoursPerDay} onChange={(event) => emit(shownAmount, safeUnit, Math.min(24, Math.max(1, Number(event.target.value) || DEFAULT_HOURS_PER_DAY)))} />
          <div>{[8, 10, 12].map((value) => <button type="button" key={value} onClick={() => emit(shownAmount, safeUnit, value)}>{value} h</button>)}</div>
        </div>
      </Field>
    </div>}
    {shownAmount != null && convertedHours != null && <p className="effort-summary">
      {shownAmount.toLocaleString("es-AR")} {effortUnitLabel(safeUnit, shownAmount)}
      {safeUnit !== "hours" ? ` × ${safeHoursPerDay.toLocaleString("es-AR")} h por día` : ""}
      {safeUnit === "weeks" ? " × 7 días" : ""}
      {` = ${convertedHours.toLocaleString("es-AR")} h para el cálculo`}
    </p>}
  </div>;
}
