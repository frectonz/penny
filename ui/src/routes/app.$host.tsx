import { useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { createFileRoute, Link, useNavigate } from '@tanstack/react-router';
import {
  ArrowLeft,
  Circle,
  Loader,
  Moon,
  Percent,
  Play,
  Server,
  Sun,
  Timer,
} from 'lucide-react';
import { useEffect, useRef } from 'react';
import { ErrorBanner } from '@/components/ErrorBanner';
import { PageContainer } from '@/components/PageContainer';
import { StatCard, StatCardSkeleton } from '@/components/StatCard';
import {
  TimeRangeSelector,
  type TimeRangeValue,
} from '@/components/TimeRangeSelector';
import { $fetch } from '@/lib/api';
import { formatMs } from '@/lib/format';
import { timeRangeSearchSchema } from '@/lib/searchSchemas';

function formatFailureRate(failures: number, total: number): string {
  if (total === 0) return '0%';
  const rate = (failures / total) * 100;
  if (rate === 0) return '0%';
  if (rate < 1) return '<1%';
  return `${Math.round(rate)}%`;
}

export const Route = createFileRoute('/app/$host')({
  component: AppDetailPage,
  validateSearch: timeRangeSearchSchema,
});

function AppDetailPage() {
  const { host } = Route.useParams();
  const { start, end } = Route.useSearch();
  const navigate = useNavigate();

  const timeRange: TimeRangeValue = { start, end };

  const handleTimeRangeChange = (value: TimeRangeValue) => {
    navigate({
      to: '/app/$host',
      params: { host },
      search: {
        start: value.start,
        end: value.end,
      },
    });
  };

  const {
    data: appOverview,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['app-overview', host, start, end],
    queryFn: () =>
      $fetch('/api/app-overview/:host', {
        params: { host },
        query: { start, end },
      }),
  });

  const {
    data: appRunsData,
    isLoading: isLoadingRuns,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ['app-runs', host, start, end],
    queryFn: ({ pageParam }) =>
      $fetch('/api/app-runs/:host', {
        params: { host },
        query: { start, end, cursor: pageParam, limit: 20 },
      }),
    initialPageParam: undefined as number | undefined,
    getNextPageParam: (lastPage) => lastPage.next_cursor ?? undefined,
  });

  const appRuns = appRunsData?.pages.flatMap((page) => page.items) ?? [];

  // Intersection observer for infinite scroll
  const loadMoreRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasNextPage && !isFetchingNextPage) {
          fetchNextPage();
        }
      },
      { threshold: 0.1 },
    );

    if (loadMoreRef.current) {
      observer.observe(loadMoreRef.current);
    }

    return () => observer.disconnect();
  }, [hasNextPage, isFetchingNextPage, fetchNextPage]);

  return (
    <PageContainer>
      {/* Header */}
      <div className="flex items-center justify-between mb-8">
        <div className="flex items-center gap-4">
          <Link
            to="/"
            search={{ start, end }}
            className="p-2 rounded-lg border border-border bg-card hover:bg-accent/10 transition-colors"
          >
            <ArrowLeft className="w-5 h-5 text-muted-foreground" />
          </Link>
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-chart-3/10 flex items-center justify-center">
              <Server className="w-5 h-5 text-chart-3" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="text-xs uppercase tracking-widest text-muted-foreground">
                  Application
                </span>
                {appOverview?.is_running && (
                  <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full bg-chart-2/10 text-chart-2">
                    <Circle className="w-2 h-2 fill-current animate-pulse" />
                    <span className="text-[10px] uppercase tracking-widest font-medium">
                      Running
                    </span>
                  </span>
                )}
              </div>
              <h1 className="text-2xl font-bold tracking-tight text-foreground">
                {host}
              </h1>
            </div>
          </div>
        </div>
        <TimeRangeSelector value={timeRange} onChange={handleTimeRangeChange} />
      </div>

      {/* Error State */}
      {error && (
        <ErrorBanner
          message={error.message || 'Failed to load application data'}
        />
      )}

      {/* Loading State */}
      {isLoading && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
          <StatCardSkeleton />
          <StatCardSkeleton />
          <StatCardSkeleton />
          <StatCardSkeleton />
          <StatCardSkeleton />
        </div>
      )}

      {/* Not Found State */}
      {!isLoading && !error && !appOverview && (
        <div className="px-5 py-4 border border-border rounded bg-card flex items-center gap-3">
          <Server className="w-5 h-5 text-muted-foreground shrink-0" />
          <p className="text-muted-foreground text-sm">
            No data found for application "{host}".
          </p>
        </div>
      )}

      {/* Stats Grid */}
      {appOverview && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
          <StatCard
            title="Total Runs"
            value={appOverview.total_runs}
            icon={Play}
            iconColor="text-accent"
            valueClassName="text-accent"
          />
          <StatCard
            title="Avg Duration"
            value={formatMs(
              appOverview.total_runs > 0
                ? Math.round(
                    appOverview.total_awake_time_ms / appOverview.total_runs,
                  )
                : 0,
            )}
            icon={Timer}
            iconColor="text-chart-1"
            valueClassName="text-chart-1"
          />
          <StatCard
            title="Awake Time"
            value={formatMs(appOverview.total_awake_time_ms)}
            icon={Sun}
            iconColor="text-chart-2"
            valueClassName="text-chart-2"
          />
          <StatCard
            title="Sleep Time"
            value={formatMs(appOverview.total_sleep_time_ms)}
            icon={Moon}
            iconColor="text-chart-4"
            valueClassName="text-chart-4"
          />
          <StatCard
            title="Failure Rate"
            value={formatFailureRate(
              appOverview.total_start_failures +
                appOverview.total_stop_failures,
              appOverview.total_runs,
            )}
            icon={Percent}
            iconColor={
              appOverview.total_start_failures +
                appOverview.total_stop_failures >
              0
                ? 'text-destructive'
                : 'text-muted-foreground'
            }
            valueClassName={
              appOverview.total_start_failures +
                appOverview.total_stop_failures >
              0
                ? 'text-destructive'
                : 'text-muted-foreground'
            }
          />
        </div>
      )}

      {/* Runs Timeline */}
      {isLoadingRuns && (
        <div className="mt-8">
          <h2 className="text-lg font-semibold text-foreground mb-4">
            Run History
          </h2>
          <div className="space-y-4">
            {[1, 2, 3].map((i) => (
              <div key={i} className="flex items-center gap-4">
                <div className="w-8 h-8 rounded-full bg-muted animate-pulse" />
                <div className="flex-1 h-16 rounded-lg bg-muted animate-pulse" />
              </div>
            ))}
          </div>
        </div>
      )}

      {appRuns.length > 0 && (
        <div className="mt-8">
          <h2 className="text-lg font-semibold text-foreground mb-4">
            Run History
          </h2>
          <div className="relative">
            {/* Timeline line */}
            <div className="absolute left-4 top-0 bottom-0 w-0.5 bg-border" />

            {/* Timeline items */}
            <div className="space-y-4">
              {appRuns.map((run, index) => {
                const runNumber = appRuns.length - index;
                const startDate = new Date(run.start_time_ms);

                return (
                  <div key={run.run_id} className="relative flex gap-4">
                    {/* Circle indicator */}
                    <div className="relative z-10 flex items-center justify-center w-8 h-8 rounded-full bg-card border-2 border-chart-3 shrink-0">
                      <Circle className="w-3 h-3 text-chart-3 fill-chart-3" />
                    </div>

                    {/* Run details */}
                    <Link
                      to="/run/$runId"
                      params={{ runId: run.run_id }}
                      className="flex-1 px-4 py-3 rounded-lg border border-border bg-card hover:bg-accent/10 transition-colors"
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <span className="text-sm font-medium text-foreground">
                            Run #{runNumber}
                          </span>
                          <span className="text-xs text-muted-foreground">
                            {startDate.toLocaleDateString()}{' '}
                            {startDate.toLocaleTimeString()}
                          </span>
                        </div>
                        <div className="flex items-center gap-1 text-chart-2">
                          <Sun className="w-4 h-4" />
                          <span className="text-sm font-medium">
                            {formatMs(run.total_awake_time_ms)}
                          </span>
                        </div>
                      </div>
                    </Link>
                  </div>
                );
              })}
            </div>

            {/* Infinite scroll trigger */}
            <div ref={loadMoreRef} className="h-4" />

            {/* Loading more indicator */}
            {isFetchingNextPage && (
              <div className="flex items-center justify-center py-4">
                <Loader className="w-5 h-5 text-muted-foreground animate-spin" />
                <span className="ml-2 text-sm text-muted-foreground">
                  Loading more...
                </span>
              </div>
            )}
          </div>
        </div>
      )}

      {!isLoadingRuns && appRuns.length === 0 && (
        <div className="mt-8">
          <h2 className="text-lg font-semibold text-foreground mb-4">
            Run History
          </h2>
          <div className="px-5 py-4 border border-border rounded bg-card flex items-center gap-3">
            <Play className="w-5 h-5 text-muted-foreground shrink-0" />
            <p className="text-muted-foreground text-sm">
              No runs recorded for this application yet.
            </p>
          </div>
        </div>
      )}
    </PageContainer>
  );
}
