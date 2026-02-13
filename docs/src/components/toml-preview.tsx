"use client";

import { Check, Copy, Download } from "lucide-react";
import { useState } from "react";

function highlightTomlLine(line: string, index: number) {
  const lineNum = (
    <span className="inline-block w-8 shrink-0 select-none text-right text-fd-muted-foreground/40 mr-4">
      {index + 1}
    </span>
  );

  if (line.startsWith("#")) {
    return (
      <div key={index} className="flex">
        {lineNum}
        <span className="text-fd-muted-foreground">{line}</span>
      </div>
    );
  }

  const table = line.match(/^(\[+)(".*?")(\..*?)?(]+)$/);
  if (table) {
    return (
      <div key={index} className="flex">
        {lineNum}
        <span>
          <span className="text-fd-muted-foreground">{table[1]}</span>
          <span className="text-fd-primary">{table[2]}</span>
          {table[3] && (
            <span className="text-fd-muted-foreground">{table[3]}</span>
          )}
          <span className="text-fd-muted-foreground">{table[4]}</span>
        </span>
      </div>
    );
  }

  const kv = line.match(/^(\w+)(\s*=\s*)(".*"|true|false|\d+|\[.*\])$/);
  if (kv) {
    const val = kv[3];
    let valClass = "text-green-600 dark:text-green-400";
    if (val === "true" || val === "false") {
      valClass = "text-fd-primary";
    } else if (/^\d+$/.test(val)) {
      valClass = "text-blue-600 dark:text-blue-400";
    } else if (val.startsWith("[")) {
      valClass = "text-green-600 dark:text-green-400";
    }

    return (
      <div key={index} className="flex">
        {lineNum}
        <span>
          <span className="text-fd-foreground">{kv[1]}</span>
          <span className="text-fd-muted-foreground">{kv[2]}</span>
          <span className={valClass}>{val}</span>
        </span>
      </div>
    );
  }

  return (
    <div key={index} className="flex">
      {lineNum}
      <span className="text-fd-foreground">{line}</span>
    </div>
  );
}

export function TomlPreview({ toml }: { toml: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(toml);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleDownload = () => {
    const blob = new Blob([toml], { type: "text/plain" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "penny.toml";
    a.click();
    URL.revokeObjectURL(url);
  };

  const lines = (toml || "# add an app to get started").split("\n");
  const hasContent = toml.length > 0;

  return (
    <div className="sticky top-20 border border-fd-border bg-fd-card">
      <div className="flex items-center justify-between border-b border-fd-border px-4 py-2">
        <div className="flex items-center gap-2">
          <div className="flex gap-1.5">
            <div className="h-2.5 w-2.5 bg-[var(--terminal-dot-red)]" />
            <div className="h-2.5 w-2.5 bg-[var(--terminal-dot-yellow)]" />
            <div className="h-2.5 w-2.5 bg-[var(--terminal-dot-green)]" />
          </div>
          <span className="text-[11px] text-fd-muted-foreground font-[family-name:var(--font-jetbrains-mono)]">
            penny.toml
          </span>
        </div>
        <div className="flex items-center gap-1">
          {hasContent && (
            <button
              type="button"
              onClick={handleDownload}
              className="inline-flex items-center gap-1.5 px-2 py-1 text-[11px] text-fd-muted-foreground transition-colors hover:bg-fd-muted hover:text-fd-foreground font-[family-name:var(--font-jetbrains-mono)] cursor-pointer"
            >
              <Download className="h-3 w-3" />
            </button>
          )}
          <button
            type="button"
            onClick={handleCopy}
            className={`inline-flex items-center gap-1.5 px-2 py-1 text-[11px] transition-colors font-[family-name:var(--font-jetbrains-mono)] cursor-pointer ${
              copied
                ? "text-green-500"
                : "text-fd-muted-foreground hover:bg-fd-muted hover:text-fd-foreground"
            }`}
          >
            {copied ? (
              <>
                <Check className="h-3 w-3" />
                copied
              </>
            ) : (
              <>
                <Copy className="h-3 w-3" />
                copy
              </>
            )}
          </button>
        </div>
      </div>
      <pre className="max-h-[calc(100vh-12rem)] overflow-auto p-4 text-[13px] leading-relaxed font-[family-name:var(--font-jetbrains-mono)]">
        <code>{lines.map((line, i) => highlightTomlLine(line, i))}</code>
      </pre>
      <div className="border-t border-fd-border px-4 py-1.5 text-[10px] text-fd-muted-foreground/50 font-[family-name:var(--font-jetbrains-mono)] flex items-center justify-between">
        <span>{lines.length} lines</span>
        <span>TOML</span>
      </div>
    </div>
  );
}
