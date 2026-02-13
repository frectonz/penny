import type { Metadata } from "next";
import { TomlBuilder } from "@/components/toml-builder";

export const metadata: Metadata = {
  title: "Builder",
  description: "Interactive penny.toml configuration builder",
};

export default function BuilderPage() {
  return (
    <div
      className="mx-auto w-full px-4 py-10"
      style={{ maxWidth: "var(--fd-layout-width, 1400px)" }}
    >
      <div className="mb-10 border-b border-fd-border pb-6">
        <div className="flex items-center gap-3 font-[family-name:var(--font-jetbrains-mono)]">
          <span className="text-fd-muted-foreground text-sm">$</span>
          <h1 className="text-xl font-bold sm:text-2xl">
            penny<span className="text-fd-primary">.</span>toml
          </h1>
        </div>
        <p className="mt-2 text-sm text-fd-muted-foreground font-[family-name:var(--font-jetbrains-mono)]">
          # interactive configuration builder
        </p>
      </div>
      <TomlBuilder />
    </div>
  );
}
