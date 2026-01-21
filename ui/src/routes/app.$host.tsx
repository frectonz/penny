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
import { z } from 'zod';
import {
  TimeRangeSelector,
  type TimeRangeValue,
} from '../components/TimeRangeSelector';
import { Skeleton } from '../components/ui/skeleton';
import { $fetch } from '../lib/api';

const searchSchema = z.object({
  start: z.number().optional(),
  end: z.number().optional(),
});

export const Route = createFileRoute('/app/$host')({
  component: AppDetailPage,
  validateSearch: searchSchema,
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
  valueColor,
}: {
  title: string;
  value: string | number;
  icon: React.ComponentType<{ className?: string }>;
  iconColor: string;
  valueColor?: string;
}) {
  return (
    <div className="p-6 border border-border rounded bg-card">
      <div className="flex items-center gap-3 mb-4">
        <div
          className={`w-12 h-12 rounded-lg ${iconColor.replace('text-', 'bg-')}/10 flex items-center justify-center`}
        >
          <Icon className={`w-6 h-6 ${iconColor}`} />
        </div>
        <span className="text-sm uppercase tracking-widest text-muted-foreground">
          {title}
        </span>
      </div>
      <p
        className={`text-4xl font-bold tabular-nums ${valueColor || 'text-foreground'}`}
      >
        {value}
      </p>
    </div>
  );
}

function StatCardSkeleton() {
  return (
    <div className="p-6 border border-border rounded bg-card">
      <div className="flex items-center gap-3 mb-4">
        <Skeleton className="w-12 h-12 rounded-lg" />
        <Skeleton className="h-4 w-24" />
      </div>
      <Skeleton className="h-10 w-32" />
    </div>
  );
}

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
    queryFn: async () => {
      const { data, error } = await $fetch('/api/app-overview/:host', {
        params: { host },
        query: { start, end },
      });
      if (error) throw error;
      return data;
    },
  });

  return (
    <div className="min-h-[calc(100vh-57px)] bg-background bg-grid p-8">
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
        <div className="px-5 py-4 mb-8 border border-destructive rounded bg-destructive/10 flex items-center gap-3">
          <AlertTriangle className="w-5 h-5 text-destructive shrink-0" />
          <p className="text-destructive text-sm">
            {error.message || 'Failed to load application data'}
          </p>
        </div>
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
            valueColor="text-chart-2"
          />
          <StatCard
            title="Sleep Time"
            value={formatMs(appOverview.total_sleep_time_ms)}
            icon={Moon}
            iconColor="text-chart-4"
            valueColor="text-chart-4"
          />
          <StatCard
            title="Start Failures"
            value={appOverview.total_start_failures}
            icon={AlertTriangle}
            iconColor="text-destructive"
            valueColor={
              appOverview.total_start_failures > 0
                ? 'text-destructive'
                : undefined
            }
          />
        </div>
      )}
    </div>
  );
}
