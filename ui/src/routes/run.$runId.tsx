import { useQuery } from '@tanstack/react-query';
import { createFileRoute, useRouter } from '@tanstack/react-router';
import { ArrowLeft, FileText, Terminal } from 'lucide-react';
import { ErrorBanner } from '@/components/ErrorBanner';
import { PageContainer } from '@/components/PageContainer';
import { Skeleton } from '@/components/ui/skeleton';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { $fetch, type LogEntry } from '@/lib/api';

export const Route = createFileRoute('/run/$runId')({
  component: RunDetailPage,
});

function RunDetailPage() {
  const { runId } = Route.useParams();
  const router = useRouter();

  const {
    data: runLogs,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['run-logs', runId],
    queryFn: () =>
      $fetch('/api/run-logs/:run_id', {
        params: { run_id: runId },
      }),
  });

  return (
    <PageContainer>
      {/* Header */}
      <div className="flex items-center gap-4 mb-8">
        <button
          type="button"
          onClick={() => router.history.back()}
          className="p-2 rounded-lg border border-border bg-card hover:bg-accent/10 transition-colors"
        >
          <ArrowLeft className="w-5 h-5 text-muted-foreground" />
        </button>
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-chart-3/10 flex items-center justify-center">
            <Terminal className="w-5 h-5 text-chart-3" />
          </div>
          <div>
            <span className="text-xs uppercase tracking-widest text-muted-foreground block">
              Run Logs
            </span>
            <h1 className="text-lg font-bold tracking-tight text-foreground font-mono">
              {runId}
            </h1>
          </div>
        </div>
      </div>

      {/* Error State */}
      {error && (
        <ErrorBanner message={error.message || 'Failed to load run logs'} />
      )}

      {/* Loading State */}
      {isLoading && (
        <div className="space-y-4">
          <Skeleton className="h-9 w-48" />
          <Skeleton className="h-96 w-full" />
        </div>
      )}

      {/* Not Found State */}
      {!isLoading && !error && !runLogs && (
        <div className="px-5 py-4 border border-border rounded bg-card flex items-center gap-3">
          <Terminal className="w-5 h-5 text-muted-foreground shrink-0" />
          <p className="text-muted-foreground text-sm">
            No logs found for this run.
          </p>
        </div>
      )}

      {/* Logs View */}
      {runLogs && (
        <Tabs defaultValue="stdout">
          <TabsList>
            <TabsTrigger value="stdout" className="gap-2">
              <FileText className="w-4 h-4" />
              stdout
              <span className="text-xs text-muted-foreground">
                ({runLogs.stdout.length})
              </span>
            </TabsTrigger>
            <TabsTrigger value="stderr" className="gap-2">
              <Terminal className="w-4 h-4" />
              stderr
              <span className="text-xs text-muted-foreground">
                ({runLogs.stderr.length})
              </span>
            </TabsTrigger>
          </TabsList>

          <TabsContent value="stdout">
            <LogViewer entries={runLogs.stdout} emptyMessage="No stdout logs" />
          </TabsContent>

          <TabsContent value="stderr">
            <LogViewer entries={runLogs.stderr} emptyMessage="No stderr logs" />
          </TabsContent>
        </Tabs>
      )}
    </PageContainer>
  );
}

function LogViewer({
  entries,
  emptyMessage,
}: {
  entries: LogEntry[];
  emptyMessage: string;
}) {
  if (entries.length === 0) {
    return (
      <div className="px-5 py-4 border border-border rounded bg-card flex items-center gap-3">
        <FileText className="w-5 h-5 text-muted-foreground shrink-0" />
        <p className="text-muted-foreground text-sm">{emptyMessage}</p>
      </div>
    );
  }

  return (
    <div className="border border-border rounded bg-card overflow-hidden">
      <div className="overflow-x-auto">
        <table className="w-full text-sm font-mono">
          <thead>
            <tr className="border-b border-border bg-muted/50">
              <th className="px-4 py-2 text-left text-xs font-medium text-muted-foreground w-48">
                Timestamp
              </th>
              <th className="px-4 py-2 text-left text-xs font-medium text-muted-foreground">
                Message
              </th>
            </tr>
          </thead>
          <tbody>
            {entries.map((entry, index) => {
              const timestamp = new Date(entry.timestamp);
              return (
                <tr
                  key={`${entry.timestamp}-${index}`}
                  className="border-b border-border last:border-b-0 hover:bg-muted/30"
                >
                  <td className="px-4 py-2 text-muted-foreground whitespace-nowrap">
                    {timestamp.toLocaleTimeString()}.
                    {String(timestamp.getMilliseconds()).padStart(3, '0')}
                  </td>
                  <td className="px-4 py-2 text-foreground whitespace-pre-wrap break-all">
                    {entry.line}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
