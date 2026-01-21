import { useQuery } from '@tanstack/react-query';
import { createFileRoute, Link, useNavigate } from '@tanstack/react-router';
import {
  AlertTriangle,
  ArrowLeft,
  Moon,
  Play,
  Server,
  Sun,
} from 'lucide-react';
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
              <span className="text-xs uppercase tracking-widest text-muted-foreground block">
                Application
              </span>
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
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
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
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <StatCard
            title="Total Runs"
            value={appOverview.total_runs}
            icon={Play}
            iconColor="text-accent"
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
            title="Start Failures"
            value={appOverview.total_start_failures}
            icon={AlertTriangle}
            iconColor="text-destructive"
            valueClassName={
              appOverview.total_start_failures > 0
                ? 'text-destructive'
                : undefined
            }
          />
        </div>
      )}
    </PageContainer>
  );
}
