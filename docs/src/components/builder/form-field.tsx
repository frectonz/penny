"use client";

import { ChevronRight } from "lucide-react";
import { type ReactNode, useState } from "react";

export function FormField({
  label,
  help,
  error,
  children,
}: {
  label: string;
  help?: string;
  error?: string;
  children: ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <span className="block text-xs font-medium text-fd-muted-foreground font-[family-name:var(--font-jetbrains-mono)] uppercase tracking-wider">
        {label}
      </span>
      {children}
      {help && !error && (
        <p className="text-xs text-fd-muted-foreground/70 font-[family-name:var(--font-jetbrains-mono)]">
          # {help}
        </p>
      )}
      {error && (
        <p className="text-xs text-red-500 font-[family-name:var(--font-jetbrains-mono)]">
          ! {error}
        </p>
      )}
    </div>
  );
}

export function FormInput({
  label,
  help,
  error,
  ...props
}: {
  label: string;
  help?: string;
  error?: string;
} & React.InputHTMLAttributes<HTMLInputElement>) {
  return (
    <FormField label={label} help={help} error={error}>
      <input
        className="w-full border border-fd-border bg-fd-background px-3 py-2 text-sm font-[family-name:var(--font-jetbrains-mono)] outline-none transition-colors focus:border-fd-primary focus:bg-fd-muted/30 placeholder:text-fd-muted-foreground/40"
        {...props}
      />
    </FormField>
  );
}

export function FormToggle({
  label,
  help,
  checked,
  onChange,
}: {
  label: string;
  help?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className="flex w-full items-center justify-between gap-4 group cursor-pointer"
    >
      <div className="text-left">
        <span className="text-xs font-medium text-fd-muted-foreground font-[family-name:var(--font-jetbrains-mono)] uppercase tracking-wider">
          {label}
        </span>
        {help && (
          <p className="text-xs text-fd-muted-foreground/70 font-[family-name:var(--font-jetbrains-mono)]">
            # {help}
          </p>
        )}
      </div>
      <div
        className={`shrink-0 border px-2 py-0.5 text-[10px] font-bold font-[family-name:var(--font-jetbrains-mono)] tracking-wider transition-colors ${
          checked
            ? "border-fd-primary bg-fd-primary/10 text-fd-primary"
            : "border-fd-border bg-fd-muted text-fd-muted-foreground"
        }`}
      >
        {checked ? "ON" : "OFF"}
      </div>
    </button>
  );
}

export function FormSegment({
  label,
  options,
  value,
  onChange,
}: {
  label: string;
  options: { value: string; label: string }[];
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <div className="space-y-1.5">
      <span className="block text-xs font-medium text-fd-muted-foreground font-[family-name:var(--font-jetbrains-mono)] uppercase tracking-wider">
        {label}
      </span>
      <div className="flex border border-fd-border">
        {options.map((opt) => (
          <button
            key={opt.value}
            type="button"
            onClick={() => onChange(opt.value)}
            className={`flex-1 px-3 py-1.5 text-xs font-medium font-[family-name:var(--font-jetbrains-mono)] transition-colors cursor-pointer ${
              value === opt.value
                ? "bg-fd-primary text-white"
                : "bg-fd-background text-fd-muted-foreground hover:bg-fd-muted hover:text-fd-foreground"
            }`}
          >
            {opt.label}
          </button>
        ))}
      </div>
    </div>
  );
}

export function FormSection({
  title,
  defaultOpen = false,
  children,
}: {
  title: string;
  defaultOpen?: boolean;
  children: ReactNode;
}) {
  const [open, setOpen] = useState(defaultOpen);

  return (
    <div className="border border-fd-border">
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="flex w-full items-center gap-2 px-4 py-3 text-left transition-colors hover:bg-fd-muted/50 cursor-pointer"
      >
        <ChevronRight
          className={`h-3 w-3 text-fd-muted-foreground transition-transform ${open ? "rotate-90" : ""}`}
        />
        <span className="text-xs font-semibold font-[family-name:var(--font-jetbrains-mono)] uppercase tracking-wider text-fd-muted-foreground">
          {title}
        </span>
      </button>
      {open && (
        <div className="space-y-4 border-t border-fd-border px-4 py-4">
          {children}
        </div>
      )}
    </div>
  );
}
