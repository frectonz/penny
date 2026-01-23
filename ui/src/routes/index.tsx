import { useQuery } from '@tanstack/react-query';
import { createFileRoute, Link, useNavigate } from '@tanstack/react-router';
import {
  Activity,
  AlertTriangle,
  Circle,
  Layers,
  Moon,
  OctagonX,
  Play,
  Server,
  Sun,
  Timer,
} from 'lucide-react';
import { ErrorBanner } from '@/components/ErrorBanner';
import { PageContainer } from '@/components/PageContainer';
import { StatCard, StatCardSkeleton } from '@/components/StatCard';
import {
  TimeRangeSelector,
  type TimeRangeValue,
} from '@/components/TimeRangeSelector';
import { Skeleton } from '@/components/ui/skeleton';
import { $fetch, type AppOverview } from '@/lib/api';
import { formatMs } from '@/lib/format';
import { timeRangeSearchSchema } from '@/lib/searchSchemas';

export const Route = createFileRoute('/')({
  component: App,
  validateSearch: timeRangeSearchSchema,
});

function AppCard({
  app,
  timeRange,
}: {
  app: AppOverview;
  timeRange: TimeRangeValue;
}) {
  return (
    <Link
      to="/app/$host"
      params={{ host: app.host }}
      search={{ start: timeRange.start, end: timeRange.end }}
      className="block p-5 border border-border rounded bg-card hover:border-accent/50 transition-colors h-full cursor-pointer"
    >
      {/* Header with host */}
      <div className="flex items-center gap-3 mb-5 pb-4 border-b border-border">
        <div className="w-10 h-10 rounded bg-chart-3/10 flex items-center justify-center shrink-0">
          <Server className="w-5 h-5 text-chart-3" />
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-[10px] uppercase tracking-widest text-muted-foreground">
              Host
            </span>
            {app.is_running && (
              <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full bg-chart-2/10 text-chart-2">
                <Circle className="w-2 h-2 fill-current animate-pulse" />
                <span className="text-[10px] uppercase tracking-widest font-medium">
                  Running
                </span>
              </span>
            )}
          </div>
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
            <p className="text-lg font-semibold text-accent tabular-nums">
              {app.total_runs}
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

        {/* Failures */}
        <div className="flex items-center gap-3">
          <div
            className={`w-8 h-8 rounded flex items-center justify-center shrink-0 ${
              app.total_start_failures + app.total_stop_failures > 0
                ? 'bg-destructive/10'
                : 'bg-muted'
            }`}
          >
            <AlertTriangle
              className={`w-4 h-4 ${
                app.total_start_failures + app.total_stop_failures > 0
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
                app.total_start_failures + app.total_stop_failures > 0
                  ? 'text-destructive'
                  : 'text-muted-foreground'
              }`}
            >
              {app.total_start_failures}/{app.total_stop_failures}
            </p>
          </div>
        </div>
      </div>
    </Link>
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
  const { start, end } = Route.useSearch();
  const navigate = useNavigate();

  const timeRange: TimeRangeValue = { start, end };

  const handleTimeRangeChange = (value: TimeRangeValue) => {
    navigate({
      to: '/',
      search: {
        start: value.start,
        end: value.end,
      },
    });
  };

  const {
    data: totalOverview,
    isLoading: isTotalLoading,
    error: totalError,
  } = useQuery({
    queryKey: ['total-overview', start, end],
    queryFn: () => $fetch('/api/total-overview', { query: { start, end } }),
  });

  const {
    data: appsOverview,
    isLoading: isAppsLoading,
    error: appsError,
  } = useQuery({
    queryKey: ['apps-overview', start, end],
    queryFn: () => $fetch('/api/apps-overview', { query: { start, end } }),
  });

  const error = totalError || appsError;

  return (
    <PageContainer>
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-3">
          <Activity className="w-6 h-6 text-accent" />
          <h1 className="text-2xl font-bold tracking-tight text-foreground">
            Dashboard
          </h1>
        </div>
        <TimeRangeSelector value={timeRange} onChange={handleTimeRangeChange} />
      </div>

      {error && <ErrorBanner message={`Error: ${error.message}`} />}

      {/* Stats Cards Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {isTotalLoading ? (
          <>
            <StatCardSkeleton />
            <StatCardSkeleton />
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
              valueClassName="text-accent"
            />
            <StatCard
              title="Avg Duration"
              value={formatMs(
                totalOverview.total_runs > 0
                  ? Math.round(
                      totalOverview.total_awake_time_ms /
                        totalOverview.total_runs,
                    )
                  : 0,
              )}
              icon={Timer}
              iconColor="text-chart-1"
              valueClassName="text-chart-1"
            />
            <StatCard
              title="Awake Time"
              value={formatMs(totalOverview.total_awake_time_ms)}
              icon={Sun}
              iconColor="text-chart-2"
              valueClassName="text-chart-2"
            />
            <StatCard
              title="Sleep Time"
              value={formatMs(totalOverview.total_sleep_time_ms)}
              icon={Moon}
              iconColor="text-chart-4"
              valueClassName="text-chart-4"
            />
            <StatCard
              title="Start Failures"
              value={totalOverview.total_start_failures}
              icon={AlertTriangle}
              iconColor={
                totalOverview.total_start_failures > 0
                  ? 'text-destructive'
                  : 'text-muted-foreground'
              }
              valueClassName={
                totalOverview.total_start_failures > 0
                  ? 'text-destructive'
                  : 'text-muted-foreground'
              }
            />
            <StatCard
              title="Stop Failures"
              value={totalOverview.total_stop_failures}
              icon={OctagonX}
              iconColor={
                totalOverview.total_stop_failures > 0
                  ? 'text-destructive'
                  : 'text-muted-foreground'
              }
              valueClassName={
                totalOverview.total_stop_failures > 0
                  ? 'text-destructive'
                  : 'text-muted-foreground'
              }
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
              <AppCard key={app.host} app={app} timeRange={timeRange} />
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
    </PageContainer>
  );
}
