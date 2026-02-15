"use client";

import { Check, Copy } from "lucide-react";
import { useState } from "react";

const command = "curl -LsSf https://pennyproxy.com/install.sh | sh";

export function CopyInstallCommand() {
  const [copied, setCopied] = useState(false);

  function handleCopy() {
    navigator.clipboard.writeText(command).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }

  return (
    <div className="flex items-center gap-3 overflow-hidden border border-[var(--landing-border)] bg-[var(--landing-bg-elevated)]/60 px-4 py-2 backdrop-blur-sm">
      <span className="flex-none font-[family-name:var(--font-jetbrains-mono)] text-xs text-[var(--landing-text-faint)]">
        $
      </span>
      <div className="relative min-w-0 flex-1">
        <code className="block whitespace-nowrap overflow-hidden font-[family-name:var(--font-jetbrains-mono)] text-xs text-[var(--landing-text-muted)]">
          {command}
        </code>
        <div
          className="pointer-events-none absolute right-0 top-0 h-full w-12"
          style={{
            background:
              "linear-gradient(to right, transparent, var(--landing-bg-elevated))",
          }}
        />
      </div>
      <button
        type="button"
        onClick={handleCopy}
        className="flex-none cursor-pointer text-[var(--landing-text-faint)] transition-colors hover:text-[var(--landing-text)]"
        aria-label="Copy install command"
      >
        {copied ? (
          <Check className="h-3.5 w-3.5 text-[#22c55e]" />
        ) : (
          <Copy className="h-3.5 w-3.5" />
        )}
      </button>
    </div>
  );
}
