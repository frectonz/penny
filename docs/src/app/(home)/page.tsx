import {
  ArrowRight,
  BarChart3,
  FileText,
  Link2,
  ShieldCheck,
  Timer,
  Zap,
} from "lucide-react";
import Link from "next/link";
import { CopyInstallCommand } from "@/components/copy-install-command";
import { HowItWorksAnimation } from "@/components/how-it-works-animation";
import { PennyLogo } from "@/components/penny-logo";

const features = [
  {
    icon: Zap,
    title: "On-Demand Start/Stop",
    description: "Apps start on first request and stop after idle timeout.",
  },
  {
    icon: Timer,
    title: "Adaptive Wait",
    description: "Idle timeout adjusts based on traffic patterns.",
  },
  {
    icon: ShieldCheck,
    title: "Auto TLS",
    description: "Automatic HTTPS via Let's Encrypt. Zero config.",
  },
  {
    icon: FileText,
    title: "Cold Start Pages",
    description: "Show a loading page while your app wakes up.",
  },
  {
    icon: Link2,
    title: "Related Warm-up",
    description: "Pre-warm related apps when traffic arrives.",
  },
  {
    icon: BarChart3,
    title: "Dashboard",
    description: "Built-in metrics, uptime tracking, and logs.",
  },
];

const heroConfig = `["myapp.example.com"]
address = "127.0.0.1:3001"
command = "node server.js"
health_check = "/"

["api.example.com"]
address = "127.0.0.1:8080"
command = "./api-server"
health_check = "/health"

["blog.example.com"]
address = "127.0.0.1:4000"
command = "hugo server"
health_check = "/"`;

const quickStartConfig = `["myapp.example.com"]
address = "127.0.0.1:3001"
wait_period = "10m"
health_check = "/"
command = "node server.js"`;

function TomlLine({ line }: { line: string }) {
  const table = line.match(/^(\[)(".*?")(])$/);
  if (table) {
    return (
      <div>
        <span className="text-[var(--terminal-punct)]">{table[1]}</span>
        <span className="text-[var(--terminal-table-name)]">{table[2]}</span>
        <span className="text-[var(--terminal-punct)]">{table[3]}</span>
      </div>
    );
  }
  const kv = line.match(/^(\w+)(\s*=\s*)(".*")$/);
  if (kv) {
    return (
      <div>
        <span className="text-[var(--terminal-key)]">{kv[1]}</span>
        <span className="text-[var(--terminal-punct)]">{kv[2]}</span>
        <span className="text-[var(--terminal-value)]">{kv[3]}</span>
      </div>
    );
  }
  return <div className="text-[var(--terminal-key)]">{line}</div>;
}

function TomlBlock({ code }: { code: string }) {
  return (
    <>
      {code.split("\n").map((line, i) => {
        if (line === "") return <div key={i} className="h-3" />;
        return <TomlLine key={i} line={line} />;
      })}
    </>
  );
}

function TerminalWindow({
  title,
  children,
  step,
}: {
  title: string;
  children: React.ReactNode;
  step?: { number: number; color: string };
}) {
  return (
    <div className="overflow-hidden border border-[var(--terminal-border)] shadow-2xl shadow-black/20 dark:shadow-black/50">
      <div className="flex items-center gap-2.5 bg-[var(--terminal-header)] px-4 py-2.5 border-b border-[var(--terminal-border-inner)]">
        {step ? (
          <span
            className="flex h-5 w-5 items-center justify-center font-[family-name:var(--font-jetbrains-mono)] text-[10px] font-bold"
            style={{ backgroundColor: `${step.color}15`, color: step.color }}
          >
            {step.number}
          </span>
        ) : (
          <div className="flex gap-1.5">
            <div className="h-2.5 w-2.5 bg-[var(--terminal-dot-red)]" />
            <div className="h-2.5 w-2.5 bg-[var(--terminal-dot-yellow)]" />
            <div className="h-2.5 w-2.5 bg-[var(--terminal-dot-green)]" />
          </div>
        )}
        <span className="text-[11px] text-[var(--terminal-title)] font-[family-name:var(--font-jetbrains-mono)]">
          {title}
        </span>
      </div>
      <div className="bg-[var(--terminal-bg)]">{children}</div>
    </div>
  );
}

export default function HomePage() {
  return (
    <main className="bg-[var(--landing-bg)]">
      {/* ─── Hero ─── */}
      <section className="relative overflow-hidden">
        <div
          className="absolute inset-0"
          style={{
            backgroundImage:
              "radial-gradient(circle, var(--landing-grid) 1px, transparent 1px)",
            backgroundSize: "32px 32px",
          }}
        />

        <div className="absolute right-[-5%] top-1/2 -translate-y-1/2 pointer-events-none select-none opacity-[0.03]">
          <PennyLogo size={600} color="var(--landing-text)" />
        </div>

        <div className="relative mx-auto max-w-6xl px-6 pt-24 pb-20 sm:pt-32 sm:pb-28">
          <div className="grid grid-cols-1 items-center gap-12 lg:grid-cols-2 lg:gap-16">
            <div>
              <a
                href="https://github.com/frectonz/penny/releases"
                target="_blank"
                rel="noopener noreferrer"
                className="mb-10 inline-flex items-center gap-2 border border-[var(--landing-border)] bg-[var(--landing-bg-elevated)]/80 px-3 py-1 text-xs text-[var(--landing-text-muted)] backdrop-blur-sm transition-colors hover:text-[var(--landing-text)]"
              >
                <span className="h-1.5 w-1.5 bg-[#22c55e]" />
                v0.0.16
                <ArrowRight className="h-3 w-3" />
              </a>

              <h1 className="mb-4 font-[family-name:var(--font-jetbrains-mono)] text-5xl font-bold tracking-tighter text-[var(--landing-text)] sm:text-6xl md:text-7xl lg:text-8xl">
                Penny
              </h1>

              <p className="mb-6 font-[family-name:var(--font-jetbrains-mono)] text-lg text-[#f97316] sm:text-xl md:text-2xl">
                Serverless for your servers.
              </p>
              <p className="mb-8 max-w-lg text-base leading-relaxed text-[var(--landing-text-muted)] sm:text-lg">
                A reverse proxy that starts your apps on demand and kills them
                when idle. Ten side projects, one VPS, zero waste.
              </p>

              <div className="mb-8 flex flex-wrap gap-3">
                <Link
                  href="/docs"
                  className="group inline-flex items-center gap-2 bg-[#f97316] px-6 py-2.5 text-sm font-semibold text-white transition-all hover:bg-[#ea580c]"
                >
                  Get Started
                  <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" />
                </Link>
                <a
                  href="https://github.com/frectonz/penny"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="inline-flex items-center gap-2 border border-[var(--landing-border)] px-6 py-2.5 text-sm font-semibold text-[var(--landing-text-muted)] transition-colors hover:border-[var(--landing-border-subtle)] hover:text-[var(--landing-text)]"
                >
                  GitHub
                </a>
              </div>

              <CopyInstallCommand />
            </div>

            <div className="lg:translate-y-8 lg:translate-x-4">
              <TerminalWindow title="penny.toml">
                <pre className="overflow-x-auto p-5 font-[family-name:var(--font-jetbrains-mono)] text-[13px] leading-relaxed">
                  <code>
                    <TomlBlock code={heroConfig} />
                  </code>
                </pre>
              </TerminalWindow>
            </div>
          </div>
        </div>

        <div
          className="h-px"
          style={{
            background:
              "linear-gradient(to right, transparent, var(--landing-border-subtle), transparent)",
          }}
        />
      </section>

      {/* ─── Video ─── */}
      <section className="py-20 sm:py-28">
        <div className="mx-auto max-w-4xl px-6">
          <div className="overflow-hidden border border-[var(--landing-border)] shadow-2xl shadow-black/10 dark:shadow-black/30">
            <div className="aspect-video">
              <iframe
                className="h-full w-full"
                src="https://www.youtube.com/embed/TnC87BvWymc"
                title="Penny demo"
                allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture"
                allowFullScreen
              />
            </div>
          </div>
        </div>

        <div className="mx-auto mt-20 max-w-5xl px-6 sm:mt-28">
          <div
            className="h-px"
            style={{
              background:
                "linear-gradient(to right, transparent, var(--landing-border-subtle), transparent)",
            }}
          />
        </div>
      </section>

      {/* ─── How It Works ─── */}
      <section className="py-20 sm:py-28">
        <div className="mx-auto max-w-5xl px-6">
          <p className="mb-3 font-[family-name:var(--font-jetbrains-mono)] text-xs font-medium uppercase tracking-[0.2em] text-[#f97316]">
            How It Works
          </p>
          <h2 className="mb-4 text-2xl font-bold tracking-tight text-[var(--landing-text)] sm:text-3xl">
            Request lifecycle
          </h2>
          <p className="mb-10 max-w-lg text-[var(--landing-text-muted)]">
            Each request resets the idle timer. Traffic keeps the app alive.
            Silence kills it.
          </p>
          <div className="overflow-hidden border border-[var(--landing-border)] shadow-lg shadow-black/5 dark:shadow-black/20">
            <HowItWorksAnimation />
          </div>
        </div>

        <div className="mx-auto mt-20 max-w-5xl px-6 sm:mt-28">
          <div
            className="h-px"
            style={{
              background:
                "linear-gradient(to right, transparent, var(--landing-border-subtle), transparent)",
            }}
          />
        </div>
      </section>

      {/* ─── Features ─── */}
      <section className="py-20 sm:py-28">
        <div className="mx-auto max-w-5xl px-6">
          <p className="mb-3 font-[family-name:var(--font-jetbrains-mono)] text-xs font-medium uppercase tracking-[0.2em] text-[#f97316]">
            Built-in
          </p>
          <h2 className="mb-12 text-2xl font-bold tracking-tight text-[var(--landing-text)] sm:text-3xl">
            Everything you need
          </h2>
          <div className="grid grid-cols-1 gap-x-8 gap-y-10 sm:grid-cols-2 lg:grid-cols-3">
            {features.map((f) => (
              <div key={f.title}>
                <f.icon
                  className="mb-3 h-5 w-5 text-[#f97316]"
                  strokeWidth={1.5}
                />
                <h3 className="mb-1.5 font-[family-name:var(--font-jetbrains-mono)] text-sm font-semibold text-[var(--landing-text)]">
                  {f.title}
                </h3>
                <p className="text-sm leading-relaxed text-[var(--landing-text-muted)]">
                  {f.description}
                </p>
              </div>
            ))}
          </div>
        </div>

        <div className="mx-auto mt-20 max-w-5xl px-6 sm:mt-28">
          <div
            className="h-px"
            style={{
              background:
                "linear-gradient(to right, transparent, var(--landing-border-subtle), transparent)",
            }}
          />
        </div>
      </section>

      {/* ─── Quick Start ─── */}
      <section className="py-20 sm:py-28">
        <div className="mx-auto max-w-3xl px-6">
          <div className="mb-12 text-center">
            <p className="mb-3 font-[family-name:var(--font-jetbrains-mono)] text-xs font-medium uppercase tracking-[0.2em] text-[#f97316]">
              Get running in seconds
            </p>
            <h2 className="text-2xl font-bold tracking-tight text-[var(--landing-text)] sm:text-3xl">
              Quick Start
            </h2>
          </div>

          <div className="space-y-4">
            <TerminalWindow
              title="penny.toml"
              step={{ number: 1, color: "#f97316" }}
            >
              <pre className="overflow-x-auto p-5 font-[family-name:var(--font-jetbrains-mono)] text-[13px] leading-relaxed">
                <code>
                  <TomlBlock code={quickStartConfig} />
                </code>
              </pre>
            </TerminalWindow>

            <TerminalWindow
              title="terminal"
              step={{ number: 2, color: "#22c55e" }}
            >
              <pre className="p-5 font-[family-name:var(--font-jetbrains-mono)] text-[13px]">
                <code>
                  <span className="text-[var(--terminal-punct)]">$ </span>
                  <span className="text-[var(--terminal-text)]">
                    penny serve penny.toml
                  </span>
                </code>
              </pre>
            </TerminalWindow>
          </div>

          <p className="mt-8 text-center text-sm text-[var(--landing-text-muted)]">
            That&apos;s it. Requests start the app, traffic keeps it alive,
            silence kills it.
          </p>
        </div>
      </section>

      {/* ─── Footer ─── */}
      <footer className="relative">
        <div
          className="h-px"
          style={{
            background:
              "linear-gradient(to right, transparent, var(--landing-border-subtle), transparent)",
          }}
        />
        <div className="mx-auto max-w-5xl px-6 py-10">
          <div className="flex flex-col items-center justify-between gap-6 sm:flex-row">
            <div className="flex items-center gap-2.5">
              <PennyLogo size={20} color="#f97316" />
              <span className="font-[family-name:var(--font-jetbrains-mono)] text-sm font-semibold text-[var(--landing-text)]">
                Penny
              </span>
            </div>
            <div className="flex items-center gap-8 font-[family-name:var(--font-jetbrains-mono)] text-xs text-[var(--landing-text-faint)]">
              <a
                href="https://github.com/frectonz/penny"
                target="_blank"
                rel="noopener noreferrer"
                className="transition-colors hover:text-[var(--landing-text)]"
              >
                GitHub
              </a>
              <Link
                href="/docs"
                className="transition-colors hover:text-[var(--landing-text)]"
              >
                Docs
              </Link>
              <Link
                href="/builder"
                className="transition-colors hover:text-[var(--landing-text)]"
              >
                Builder
              </Link>
              <span>MIT License</span>
            </div>
          </div>
          <div className="mt-8 text-center font-[family-name:var(--font-jetbrains-mono)] text-[11px] text-[var(--landing-text-faint)]">
            Built with{" "}
            <a
              href="https://github.com/cloudflare/pingora"
              target="_blank"
              rel="noopener noreferrer"
              className="transition-colors hover:text-[#f97316]"
            >
              Pingora
            </a>
          </div>
        </div>
      </footer>
    </main>
  );
}
