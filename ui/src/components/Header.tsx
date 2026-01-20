import { useQuery } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { $fetch } from '../lib/api';
import { Skeleton } from './ui/skeleton';

export default function Header() {
  const { data, isLoading } = useQuery({
    queryKey: ['version'],
    queryFn: async () => {
      const { data, error } = await $fetch('/api/version');
      if (error) throw error;
      return data;
    },
  });

  return (
    <header className="px-6 py-4 flex items-center justify-between bg-background border-b border-border">
      <h1 className="text-lg font-semibold tracking-wider">
        <Link
          to="/"
          className="text-foreground hover:text-accent transition-colors"
        >
          PENNY.
        </Link>
      </h1>
      <nav className="flex items-center gap-8">
        <Link
          to="/"
          className="text-sm uppercase tracking-widest text-muted-foreground hover:text-foreground transition-colors"
        >
          Home
        </Link>
        {isLoading ? (
          <Skeleton className="h-4 w-12" />
        ) : data ? (
          <span className="text-xs text-muted-foreground">v{data.version}</span>
        ) : null}
      </nav>
    </header>
  );
}
