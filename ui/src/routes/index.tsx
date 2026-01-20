import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';
import type { LucideIcon } from 'lucide-react';
import {
  Activity,
  AlertTriangle,
  Layers,
  Moon,
  Play,
  Server,
  Sun,
} from 'lucide-react';
import { Skeleton } from '../components/ui/skeleton';
import { $fetch, type AppOverview } from '../lib/api';

export const Route = createFileRoute('/')({
  component: App,
});

function formatMs(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  }

  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) {
    return `${seconds}s`;
  }

  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    const remainingSeconds = seconds % 60;
    return remainingSeconds > 0
      ? `${minutes}m ${remainingSeconds}s`
      : `${minutes}m`;
  }

  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return remainingMinutes > 0 ? `${hours}h ${remainingMinutes}m` : `${hours}h`;
}

function StatCard({
  title,
  value,
  icon: Icon,
  iconColor,
}: {
  title: string;
  value: string | number;
  icon: LucideIcon;
  iconColor: string;
}) {
  return (
    <div className="p-5 border border-border rounded bg-card h-full">
      <div className="flex items-center gap-2 mb-3">
        <Icon className={`w-4 h-4 ${iconColor} shrink-0`} />
        <span className="text-xs uppercase tracking-widest text-muted-foreground">
          {title}
        </span>
      </div>
      <p className="text-3xl font-semibold text-foreground tabular-nums">
        {value}
      </p>
    </div>
  );
}

function StatCardSkeleton() {
  return (
    <div className="p-5 border border-border rounded bg-card h-full">
      <div className="flex items-center gap-2 mb-3">
        <Skeleton className="h-4 w-4 rounded shrink-0" />
        <Skeleton className="h-4 w-24" />
      </div>
      <Skeleton className="h-9 w-20" />
    </div>
  );
}

function AppCard({ app }: { app: AppOverview }) {
  return (
    <div className="p-5 border border-border rounded bg-card hover:border-accent/50 transition-colors h-full">
      {/* Header with host */}
      <div className="flex items-center gap-3 mb-5 pb-4 border-b border-border">
        <div className="w-10 h-10 rounded bg-chart-3/10 flex items-center justify-center shrink-0">
          <Server className="w-5 h-5 text-chart-3" />
        </div>
        <div className="min-w-0 flex-1">
          <span className="text-[10px] uppercase tracking-widest text-muted-foreground block">
            Host
          </span>
          <p className="text-base font-semibold text-foreground truncate">
            {app.host}
          </p>
        </div>
      </div>

      {/* Stats grid */}
      <div className="grid grid-cols-2 gap-4">
        {/* Runs */}
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded bg-accent/10 flex items-center justify-center shrink-0">
            <Play className="w-4 h-4 text-accent" />
          </div>
          <div>
            <span className="text-[10px] uppercase tracking-widest text-muted-foreground block">
              Runs
            </span>
            <p className="text-lg font-semibold text-foreground tabular-nums">
              {app.total_runs}
            </p>
          </div>
        </div>

        {/* Failures */}
        <div className="flex items-center gap-3">
          <div
            className={`w-8 h-8 rounded flex items-center justify-center shrink-0 ${
              app.total_start_failures > 0 ? 'bg-destructive/10' : 'bg-muted'
            }`}
          >
            <AlertTriangle
              className={`w-4 h-4 ${
                app.total_start_failures > 0
                  ? 'text-destructive'
                  : 'text-muted-foreground'
              }`}
            />
          </div>
          <div>
            <span className="text-[10px] uppercase tracking-widest text-muted-foreground block">
              Failures
            </span>
            <p
              className={`text-lg font-semibold tabular-nums ${
                app.total_start_failures > 0
                  ? 'text-destructive'
                  : 'text-foreground'
              }`}
            >
              {app.total_start_failures}
            </p>
          </div>
        </div>

        {/* Awake Time */}
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded bg-chart-2/10 flex items-center justify-center shrink-0">
            <Sun className="w-4 h-4 text-chart-2" />
          </div>
          <div>
            <span className="text-[10px] uppercase tracking-widest text-muted-foreground block">
              Awake
            </span>
            <p className="text-lg font-semibold text-chart-2 tabular-nums">
              {formatMs(app.total_awake_time_ms)}
            </p>
          </div>
        </div>

        {/* Sleep Time */}
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded bg-chart-4/10 flex items-center justify-center shrink-0">
            <Moon className="w-4 h-4 text-chart-4" />
          </div>
          <div>
            <span className="text-[10px] uppercase tracking-widest text-muted-foreground block">
              Sleep
            </span>
            <p className="text-lg font-semibold text-chart-4 tabular-nums">
              {formatMs(app.total_sleep_time_ms)}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function AppCardSkeleton() {
  return (
    <div className="p-5 border border-border rounded bg-card h-full">
      {/* Header skeleton */}
      <div className="flex items-center gap-3 mb-5 pb-4 border-b border-border">
        <Skeleton className="w-10 h-10 rounded shrink-0" />
        <div className="flex-1">
          <Skeleton className="h-3 w-10 mb-1" />
          <Skeleton className="h-5 w-28" />
        </div>
      </div>

      {/* Stats grid skeleton */}
      <div className="grid grid-cols-2 gap-4">
        <div className="flex items-center gap-3">
          <Skeleton className="w-8 h-8 rounded shrink-0" />
          <div>
            <Skeleton className="h-3 w-12 mb-1" />
            <Skeleton className="h-5 w-14" />
          </div>
        </div>
        <div className="flex items-center gap-3">
          <Skeleton className="w-8 h-8 rounded shrink-0" />
          <div>
            <Skeleton className="h-3 w-12 mb-1" />
            <Skeleton className="h-5 w-14" />
          </div>
        </div>
        <div className="flex items-center gap-3">
          <Skeleton className="w-8 h-8 rounded shrink-0" />
          <div>
            <Skeleton className="h-3 w-12 mb-1" />
            <Skeleton className="h-5 w-14" />
          </div>
        </div>
        <div className="flex items-center gap-3">
          <Skeleton className="w-8 h-8 rounded shrink-0" />
          <div>
            <Skeleton className="h-3 w-12 mb-1" />
            <Skeleton className="h-5 w-14" />
          </div>
        </div>
      </div>
    </div>
  );
}

function AppsGridSkeleton() {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <AppCardSkeleton />
      <AppCardSkeleton />
      <AppCardSkeleton />
    </div>
  );
}

function App() {
  const {
    data: totalOverview,
    isLoading: isTotalLoading,
    error: totalError,
  } = useQuery({
    queryKey: ['total-overview'],
    queryFn: async () => {
      const { data, error } = await $fetch('/api/total-overview', {
        query: {},
      });
      if (error) throw error;
      return data;
    },
  });

  const {
    data: appsOverview,
    isLoading: isAppsLoading,
    error: appsError,
  } = useQuery({
    queryKey: ['apps-overview'],
    queryFn: async () => {
      const { data, error } = await $fetch('/api/apps-overview', {
        query: {},
      });
      if (error) throw error;
      return data;
    },
  });

  const error = totalError || appsError;

  return (
    <div className="min-h-[calc(100vh-57px)] bg-background bg-grid p-8">
      <div className="flex items-center gap-3 mb-8">
        <Activity className="w-6 h-6 text-accent" />
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          Dashboard
        </h1>
      </div>

      {error && (
        <div className="px-5 py-4 mb-8 border border-destructive rounded bg-destructive/10 flex items-center gap-3">
          <AlertTriangle className="w-5 h-5 text-destructive shrink-0" />
          <p className="text-destructive text-sm">Error: {error.message}</p>
        </div>
      )}

      {/* Stats Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {isTotalLoading ? (
          <>
            <StatCardSkeleton />
            <StatCardSkeleton />
            <StatCardSkeleton />
            <StatCardSkeleton />
          </>
        ) : totalOverview ? (
          <>
            <StatCard
              title="Total Runs"
              value={totalOverview.total_runs}
              icon={Play}
              iconColor="text-accent"
            />
            <StatCard
              title="Awake Time"
              value={formatMs(totalOverview.total_awake_time_ms)}
              icon={Sun}
              iconColor="text-chart-2"
            />
            <StatCard
              title="Sleep Time"
              value={formatMs(totalOverview.total_sleep_time_ms)}
              icon={Moon}
              iconColor="text-chart-4"
            />
            <StatCard
              title="Start Failures"
              value={totalOverview.total_start_failures}
              icon={AlertTriangle}
              iconColor="text-destructive"
            />
          </>
        ) : null}
      </div>

      {/* Apps List Section */}
      <div className="mt-10">
        <div className="flex items-center gap-3 mb-4">
          <Layers className="w-5 h-5 text-chart-3" />
          <h2 className="text-lg font-semibold tracking-tight text-foreground">
            Applications
          </h2>
        </div>
        {isAppsLoading ? (
          <AppsGridSkeleton />
        ) : appsOverview && appsOverview.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {appsOverview.map((app) => (
              <AppCard key={app.host} app={app} />
            ))}
          </div>
        ) : (
          <div className="px-5 py-4 border border-border rounded bg-card flex items-center gap-3">
            <Server className="w-5 h-5 text-muted-foreground shrink-0" />
            <p className="text-muted-foreground text-sm">
              No applications found.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
