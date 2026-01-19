import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';
import { $fetch } from '../lib/api';

export const Route = createFileRoute('/')({
  component: App,
});

function App() {
  const { data, isLoading, error } = useQuery({
    queryKey: ['version'],
    queryFn: async () => {
      const { data, error } = await $fetch('/api/version');
      if (error) throw error;
      return data;
    },
  });

  return (
    <div className="min-h-[calc(100vh-57px)] flex flex-col items-center justify-center bg-background bg-grid">
      <div className="text-center space-y-6">
        <h1 className="text-5xl font-bold tracking-tight text-foreground">
          PENNY
        </h1>
        <p className="text-muted-foreground text-sm uppercase tracking-widest">
          Application Monitor
        </p>

        <div className="mt-8 p-6 border border-border rounded bg-card/50 backdrop-blur-sm">
          {isLoading && (
            <p className="text-muted-foreground text-sm">Loading...</p>
          )}
          {error && (
            <p className="text-destructive text-sm">Error: {error.message}</p>
          )}
          {data && (
            <div className="space-y-2">
              <span className="text-muted-foreground text-xs uppercase tracking-widest">
                Version
              </span>
              <p className="text-accent text-2xl font-semibold">
                {data.version}
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
