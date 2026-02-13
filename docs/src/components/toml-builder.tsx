"use client";

import { ChevronRight, Plus, Server, X } from "lucide-react";
import { useCallback, useState } from "react";
import {
  FormInput,
  FormSection,
  FormSegment,
  FormToggle,
} from "@/components/builder/form-field";
import { TomlPreview } from "@/components/toml-preview";

interface AppConfig {
  id: string;
  hostname: string;
  address: string;
  healthCheck: string;
  commandType: "simple" | "start-end";
  command: string;
  startCommand: string;
  endCommand: string;
  waitPeriod: string;
  adaptiveWait: boolean;
  minWaitPeriod: string;
  maxWaitPeriod: string;
  lowReqPerHour: string;
  highReqPerHour: string;
  coldStartPage: boolean;
  coldStartPagePath: string;
  alsoWarm: string;
  startTimeout: string;
  stopTimeout: string;
  healthCheckInitialBackoffMs: string;
  healthCheckMaxBackoffSecs: string;
  showAdvanced: boolean;
}

interface GlobalConfig {
  apiAddress: string;
  apiDomain: string;
  databaseUrl: string;
}

interface TlsConfig {
  enabled: boolean;
  acmeEmail: string;
  staging: boolean;
  certsDir: string;
  renewalDays: string;
  renewalCheckIntervalHours: string;
}

function createApp(): AppConfig {
  return {
    id: Math.random().toString(36).slice(2, 9),
    hostname: "",
    address: "",
    healthCheck: "/",
    commandType: "simple",
    command: "",
    startCommand: "",
    endCommand: "",
    waitPeriod: "10m",
    adaptiveWait: false,
    minWaitPeriod: "5m",
    maxWaitPeriod: "30m",
    lowReqPerHour: "12",
    highReqPerHour: "300",
    coldStartPage: false,
    coldStartPagePath: "",
    alsoWarm: "",
    startTimeout: "30s",
    stopTimeout: "30s",
    healthCheckInitialBackoffMs: "10",
    healthCheckMaxBackoffSecs: "2",
    showAdvanced: false,
  };
}

function escapeToml(s: string): string {
  return s.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
}

function generateToml(
  global: GlobalConfig,
  tls: TlsConfig,
  apps: AppConfig[],
): string {
  const lines: string[] = [];

  if (global.apiAddress)
    lines.push(`api_address = "${escapeToml(global.apiAddress)}"`);
  if (global.apiDomain)
    lines.push(`api_domain = "${escapeToml(global.apiDomain)}"`);
  if (global.databaseUrl && global.databaseUrl !== "sqlite://penny.db")
    lines.push(`database_url = "${escapeToml(global.databaseUrl)}"`);

  if (tls.enabled) {
    if (lines.length > 0) lines.push("");
    lines.push("[tls]");
    lines.push("enabled = true");
    if (tls.acmeEmail)
      lines.push(`acme_email = "${escapeToml(tls.acmeEmail)}"`);
    if (tls.staging) lines.push("staging = true");
    if (tls.certsDir && tls.certsDir !== "./certs")
      lines.push(`certs_dir = "${escapeToml(tls.certsDir)}"`);
    if (tls.renewalDays && tls.renewalDays !== "30")
      lines.push(`renewal_days = ${tls.renewalDays}`);
    if (tls.renewalCheckIntervalHours && tls.renewalCheckIntervalHours !== "12")
      lines.push(
        `renewal_check_interval_hours = ${tls.renewalCheckIntervalHours}`,
      );
  }

  for (const app of apps) {
    if (!app.hostname) continue;
    if (lines.length > 0) lines.push("");

    const useTableCommand =
      app.commandType === "start-end" && app.startCommand && app.endCommand;

    lines.push(`["${escapeToml(app.hostname)}"]`);
    if (app.address) lines.push(`address = "${escapeToml(app.address)}"`);
    if (app.healthCheck)
      lines.push(`health_check = "${escapeToml(app.healthCheck)}"`);

    if (useTableCommand) {
      // will add as inline table below
    } else if (app.command) {
      lines.push(`command = "${escapeToml(app.command)}"`);
    }

    if (app.waitPeriod && app.waitPeriod !== "10m" && !app.adaptiveWait)
      lines.push(`wait_period = "${escapeToml(app.waitPeriod)}"`);

    if (app.adaptiveWait) {
      lines.push("adaptive_wait = true");
      if (app.minWaitPeriod && app.minWaitPeriod !== "5m")
        lines.push(`min_wait_period = "${escapeToml(app.minWaitPeriod)}"`);
      if (app.maxWaitPeriod && app.maxWaitPeriod !== "30m")
        lines.push(`max_wait_period = "${escapeToml(app.maxWaitPeriod)}"`);
      if (app.lowReqPerHour && app.lowReqPerHour !== "12")
        lines.push(`low_req_per_hour = ${app.lowReqPerHour}`);
      if (app.highReqPerHour && app.highReqPerHour !== "300")
        lines.push(`high_req_per_hour = ${app.highReqPerHour}`);
    }

    if (app.coldStartPage && !app.coldStartPagePath)
      lines.push("cold_start_page = true");
    if (app.coldStartPagePath)
      lines.push(
        `cold_start_page_path = "${escapeToml(app.coldStartPagePath)}"`,
      );

    if (app.alsoWarm) {
      const hosts = app.alsoWarm
        .split(",")
        .map((h) => h.trim())
        .filter(Boolean);
      if (hosts.length > 0)
        lines.push(
          `also_warm = [${hosts.map((h) => `"${escapeToml(h)}"`).join(", ")}]`,
        );
    }

    if (app.startTimeout && app.startTimeout !== "30s")
      lines.push(`start_timeout = "${escapeToml(app.startTimeout)}"`);
    if (app.stopTimeout && app.stopTimeout !== "30s")
      lines.push(`stop_timeout = "${escapeToml(app.stopTimeout)}"`);
    if (
      app.healthCheckInitialBackoffMs &&
      app.healthCheckInitialBackoffMs !== "10"
    )
      lines.push(
        `health_check_initial_backoff_ms = ${app.healthCheckInitialBackoffMs}`,
      );
    if (app.healthCheckMaxBackoffSecs && app.healthCheckMaxBackoffSecs !== "2")
      lines.push(
        `health_check_max_backoff_secs = ${app.healthCheckMaxBackoffSecs}`,
      );

    if (useTableCommand) {
      lines.push("");
      lines.push(`["${escapeToml(app.hostname)}".command]`);
      lines.push(`start = "${escapeToml(app.startCommand)}"`);
      lines.push(`end = "${escapeToml(app.endCommand)}"`);
    }
  }

  return lines.join("\n");
}

export function TomlBuilder() {
  const [globalConfig, setGlobalConfig] = useState<GlobalConfig>({
    apiAddress: "",
    apiDomain: "",
    databaseUrl: "",
  });

  const [tlsConfig, setTlsConfig] = useState<TlsConfig>({
    enabled: false,
    acmeEmail: "",
    staging: false,
    certsDir: "",
    renewalDays: "",
    renewalCheckIntervalHours: "",
  });

  const [apps, setApps] = useState<AppConfig[]>([createApp()]);
  const [activeAppId, setActiveAppId] = useState<string>(apps[0].id);

  const updateApp = useCallback((id: string, updates: Partial<AppConfig>) => {
    setApps((prev) =>
      prev.map((app) => (app.id === id ? { ...app, ...updates } : app)),
    );
  }, []);

  const removeApp = useCallback(
    (id: string) => {
      setApps((prev) => {
        const next = prev.filter((app) => app.id !== id);
        if (id === activeAppId && next.length > 0) {
          setActiveAppId(next[0].id);
        }
        return next;
      });
    },
    [activeAppId],
  );

  const addApp = useCallback(() => {
    const app = createApp();
    setApps((prev) => [...prev, app]);
    setActiveAppId(app.id);
  }, []);

  const toml = generateToml(globalConfig, tlsConfig, apps);

  return (
    <div className="flex flex-col gap-6 lg:flex-row">
      {/* Form */}
      <div className="space-y-6 min-w-0 lg:flex-1">
        {/* Apps — tabbed interface */}
        <div>
          {/* Tab bar */}
          <div className="flex items-end gap-0 border-b border-fd-border overflow-x-auto">
            {apps.map((app, index) => (
              <div key={app.id} className="flex items-center shrink-0">
                <button
                  type="button"
                  onClick={() => setActiveAppId(app.id)}
                  className={`flex items-center gap-2 px-4 py-2.5 text-xs font-medium font-[family-name:var(--font-jetbrains-mono)] transition-colors cursor-pointer border-b-2 -mb-px ${
                    activeAppId === app.id
                      ? "border-fd-primary text-fd-primary bg-fd-card"
                      : "border-transparent text-fd-muted-foreground hover:text-fd-foreground"
                  }`}
                >
                  <Server className="h-3 w-3" strokeWidth={1.5} />
                  {app.hostname || `app_${index + 1}`}
                  {apps.length > 1 && (
                    <button
                      type="button"
                      onClick={(e) => {
                        e.stopPropagation();
                        removeApp(app.id);
                      }}
                      className="ml-1 p-0.5 text-fd-muted-foreground hover:text-red-500 transition-colors cursor-pointer"
                    >
                      <X className="h-3 w-3" />
                    </button>
                  )}
                </button>
              </div>
            ))}
            <button
              type="button"
              onClick={addApp}
              className="flex items-center gap-1.5 px-3 py-2.5 text-xs text-fd-muted-foreground hover:text-fd-primary transition-colors cursor-pointer shrink-0 border-b-2 border-transparent -mb-px"
            >
              <Plus className="h-3 w-3" />
              <span className="font-[family-name:var(--font-jetbrains-mono)]">
                add
              </span>
            </button>
          </div>

          {/* Active app content */}
          {apps
            .filter((app) => app.id === activeAppId)
            .map((app) => (
              <div key={app.id} className="border border-t-0 border-fd-border">
                <div className="space-y-4 px-5 py-5">
                  <FormInput
                    label="hostname"
                    help="The domain this app serves"
                    placeholder="myapp.example.com"
                    value={app.hostname}
                    onChange={(e) =>
                      updateApp(app.id, { hostname: e.target.value })
                    }
                  />
                  <FormInput
                    label="address"
                    help="Backend address"
                    placeholder="127.0.0.1:3001"
                    value={app.address}
                    onChange={(e) =>
                      updateApp(app.id, { address: e.target.value })
                    }
                  />
                  <FormInput
                    label="health_check"
                    help="HTTP path to check if the app is ready"
                    placeholder="/"
                    value={app.healthCheck}
                    onChange={(e) =>
                      updateApp(app.id, { healthCheck: e.target.value })
                    }
                  />

                  {/* Command Type Segment */}
                  <FormSegment
                    label="command type"
                    options={[
                      { value: "simple", label: "simple" },
                      { value: "start-end", label: "start / stop" },
                    ]}
                    value={app.commandType}
                    onChange={(v) =>
                      updateApp(app.id, {
                        commandType: v as "simple" | "start-end",
                      })
                    }
                  />

                  {app.commandType === "simple" ? (
                    <FormInput
                      label="command"
                      help="Shell command to start the app"
                      placeholder="node server.js"
                      value={app.command}
                      onChange={(e) =>
                        updateApp(app.id, { command: e.target.value })
                      }
                    />
                  ) : (
                    <>
                      <FormInput
                        label="command.start"
                        placeholder="docker start myapp"
                        value={app.startCommand}
                        onChange={(e) =>
                          updateApp(app.id, { startCommand: e.target.value })
                        }
                      />
                      <FormInput
                        label="command.end"
                        placeholder="docker stop myapp"
                        value={app.endCommand}
                        onChange={(e) =>
                          updateApp(app.id, { endCommand: e.target.value })
                        }
                      />
                    </>
                  )}

                  <FormInput
                    label="wait_period"
                    help="Idle timeout before killing (default: 10m)"
                    placeholder="10m"
                    value={app.waitPeriod}
                    onChange={(e) =>
                      updateApp(app.id, { waitPeriod: e.target.value })
                    }
                  />

                  {/* Advanced Toggle */}
                  <button
                    type="button"
                    onClick={() =>
                      updateApp(app.id, { showAdvanced: !app.showAdvanced })
                    }
                    className="flex w-full items-center gap-2 border border-fd-border px-3 py-2 text-left transition-colors hover:bg-fd-muted/50 cursor-pointer"
                  >
                    <ChevronRight
                      className={`h-3 w-3 text-fd-muted-foreground transition-transform ${app.showAdvanced ? "rotate-90" : ""}`}
                    />
                    <span className="text-xs font-medium font-[family-name:var(--font-jetbrains-mono)] uppercase tracking-wider text-fd-muted-foreground">
                      Advanced
                    </span>
                  </button>

                  {app.showAdvanced && (
                    <div className="space-y-4 border-l-2 border-fd-primary/30 pl-4">
                      <FormToggle
                        label="adaptive_wait"
                        help="Dynamic idle timeout based on traffic"
                        checked={app.adaptiveWait}
                        onChange={(v) => updateApp(app.id, { adaptiveWait: v })}
                      />
                      {app.adaptiveWait && (
                        <>
                          <FormInput
                            label="min_wait_period"
                            placeholder="5m"
                            value={app.minWaitPeriod}
                            onChange={(e) =>
                              updateApp(app.id, {
                                minWaitPeriod: e.target.value,
                              })
                            }
                          />
                          <FormInput
                            label="max_wait_period"
                            placeholder="30m"
                            value={app.maxWaitPeriod}
                            onChange={(e) =>
                              updateApp(app.id, {
                                maxWaitPeriod: e.target.value,
                              })
                            }
                          />
                          <FormInput
                            label="low_req_per_hour"
                            placeholder="12"
                            type="number"
                            value={app.lowReqPerHour}
                            onChange={(e) =>
                              updateApp(app.id, {
                                lowReqPerHour: e.target.value,
                              })
                            }
                          />
                          <FormInput
                            label="high_req_per_hour"
                            placeholder="300"
                            type="number"
                            value={app.highReqPerHour}
                            onChange={(e) =>
                              updateApp(app.id, {
                                highReqPerHour: e.target.value,
                              })
                            }
                          />
                        </>
                      )}

                      <FormToggle
                        label="cold_start_page"
                        help="Show a loading page during cold starts"
                        checked={app.coldStartPage}
                        onChange={(v) =>
                          updateApp(app.id, { coldStartPage: v })
                        }
                      />
                      {app.coldStartPage && (
                        <FormInput
                          label="cold_start_page_path"
                          help="Path to custom HTML file (optional)"
                          placeholder="./loading.html"
                          value={app.coldStartPagePath}
                          onChange={(e) =>
                            updateApp(app.id, {
                              coldStartPagePath: e.target.value,
                            })
                          }
                        />
                      )}

                      <FormInput
                        label="also_warm"
                        help="Comma-separated hostnames to pre-warm"
                        placeholder="app2.example.com, app3.example.com"
                        value={app.alsoWarm}
                        onChange={(e) =>
                          updateApp(app.id, { alsoWarm: e.target.value })
                        }
                      />

                      <FormInput
                        label="start_timeout"
                        help="Max time to wait for health check (default: 30s)"
                        placeholder="30s"
                        value={app.startTimeout}
                        onChange={(e) =>
                          updateApp(app.id, { startTimeout: e.target.value })
                        }
                      />
                      <FormInput
                        label="stop_timeout"
                        help="Max time to wait for stop (default: 30s)"
                        placeholder="30s"
                        value={app.stopTimeout}
                        onChange={(e) =>
                          updateApp(app.id, { stopTimeout: e.target.value })
                        }
                      />
                      <FormInput
                        label="health_check_initial_backoff_ms"
                        help="Initial retry delay for health checks (default: 10)"
                        placeholder="10"
                        type="number"
                        value={app.healthCheckInitialBackoffMs}
                        onChange={(e) =>
                          updateApp(app.id, {
                            healthCheckInitialBackoffMs: e.target.value,
                          })
                        }
                      />
                      <FormInput
                        label="health_check_max_backoff_secs"
                        help="Max retry delay for health checks (default: 2)"
                        placeholder="2"
                        type="number"
                        value={app.healthCheckMaxBackoffSecs}
                        onChange={(e) =>
                          updateApp(app.id, {
                            healthCheckMaxBackoffSecs: e.target.value,
                          })
                        }
                      />
                    </div>
                  )}
                </div>
              </div>
            ))}
        </div>

        {/* Global Settings */}
        <FormSection title="Global Settings">
          <FormInput
            label="api_address"
            help="Address for the dashboard/API server"
            placeholder="0.0.0.0:3031"
            value={globalConfig.apiAddress}
            onChange={(e) =>
              setGlobalConfig({ ...globalConfig, apiAddress: e.target.value })
            }
          />
          <FormInput
            label="api_domain"
            help="Route dashboard through the proxy with TLS"
            placeholder="penny.example.com"
            value={globalConfig.apiDomain}
            onChange={(e) =>
              setGlobalConfig({ ...globalConfig, apiDomain: e.target.value })
            }
          />
          <FormInput
            label="database_url"
            help="SQLite database path (default: sqlite://penny.db)"
            placeholder="sqlite://penny.db"
            value={globalConfig.databaseUrl}
            onChange={(e) =>
              setGlobalConfig({ ...globalConfig, databaseUrl: e.target.value })
            }
          />
        </FormSection>

        {/* TLS Settings */}
        <FormSection title="TLS">
          <FormToggle
            label="enabled"
            help="Automatic HTTPS via Let's Encrypt"
            checked={tlsConfig.enabled}
            onChange={(v) => setTlsConfig({ ...tlsConfig, enabled: v })}
          />
          {tlsConfig.enabled && (
            <>
              <FormInput
                label="acme_email"
                help="Contact email for Let's Encrypt (required)"
                placeholder="you@example.com"
                type="email"
                value={tlsConfig.acmeEmail}
                onChange={(e) =>
                  setTlsConfig({ ...tlsConfig, acmeEmail: e.target.value })
                }
              />
              <FormToggle
                label="staging"
                help="Use Let's Encrypt staging environment for testing"
                checked={tlsConfig.staging}
                onChange={(v) => setTlsConfig({ ...tlsConfig, staging: v })}
              />
              <FormInput
                label="certs_dir"
                help="Directory to store certificates (default: ./certs)"
                placeholder="./certs"
                value={tlsConfig.certsDir}
                onChange={(e) =>
                  setTlsConfig({ ...tlsConfig, certsDir: e.target.value })
                }
              />
              <FormInput
                label="renewal_days"
                help="Renew certificates this many days before expiry (default: 30)"
                placeholder="30"
                type="number"
                value={tlsConfig.renewalDays}
                onChange={(e) =>
                  setTlsConfig({ ...tlsConfig, renewalDays: e.target.value })
                }
              />
              <FormInput
                label="renewal_check_interval_hours"
                help="How often to check for renewals (default: 12)"
                placeholder="12"
                type="number"
                value={tlsConfig.renewalCheckIntervalHours}
                onChange={(e) =>
                  setTlsConfig({
                    ...tlsConfig,
                    renewalCheckIntervalHours: e.target.value,
                  })
                }
              />
            </>
          )}
        </FormSection>
      </div>

      {/* TOML Preview */}
      <div className="min-w-0 lg:flex-1">
        <TomlPreview toml={toml} />
      </div>
    </div>
  );
}
