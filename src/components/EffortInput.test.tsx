import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { EffortInput } from "./EffortInput";

describe("EffortInput", () => {
  it("converts three eight-hour days into 24 calculation hours", () => {
    const onChange = vi.fn();
    const { rerender } = render(<EffortInput amount={null} unit="hours" hoursPerDay={8} estimatedHours={null} onChange={onChange} />);

    fireEvent.change(screen.getByLabelText("Unidad de tiempo"), { target: { value: "days" } });
    expect(onChange).toHaveBeenLastCalledWith({ amount: null, unit: "days", hoursPerDay: 8, estimatedHours: null });

    rerender(<EffortInput amount={null} unit="days" hoursPerDay={8} estimatedHours={null} onChange={onChange} />);
    fireEvent.change(screen.getByLabelText("Tiempo estimado"), { target: { value: "3" } });
    expect(onChange).toHaveBeenLastCalledWith({ amount: 3, unit: "days", hoursPerDay: 8, estimatedHours: 24 });
  });

  it("shows an existing 24-hour estimate as three eight-hour days", () => {
    const onChange = vi.fn();
    render(<EffortInput amount={null} unit="hours" hoursPerDay={8} estimatedHours={24} onChange={onChange} />);
    fireEvent.change(screen.getByLabelText("Unidad de tiempo"), { target: { value: "days" } });
    expect(onChange).toHaveBeenLastCalledWith({ amount: 3, unit: "days", hoursPerDay: 8, estimatedHours: 24 });
  });
});
