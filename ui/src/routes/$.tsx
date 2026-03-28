import { createFileRoute, Link, useNavigate } from '@tanstack/react-router';
import { AlertCircle, Home } from 'lucide-react';
import { useEffect, useState } from 'react';
import { PageContainer } from '@/components/PageContainer';

export const Route = createFileRoute('/$')({
  component: NotFound,
});

function NotFound() {
  const navigate = useNavigate();
  const [countdown, setCountdown] = useState(10);

  useEffect(() => {
    if (countdown === 0) {
      navigate({ to: '/' });
      return;
    }
    const timer = setTimeout(() => setCountdown((c) => c - 1), 1000);
    return () => clearTimeout(timer);
  }, [countdown, navigate]);

  return (
    <PageContainer>
      <div className="flex min-h-[60vh] flex-col items-center justify-center text-center">
        <div className="mb-6 flex h-20 w-20 items-center justify-center rounded-full bg-destructive/10">
          <AlertCircle className="h-10 w-10 text-destructive" />
        </div>
        <h1 className="mb-2 text-6xl font-bold text-foreground">404</h1>
        <p className="mb-2 text-xl font-medium text-foreground">
          Page not found
        </p>
        <p className="mb-8 text-muted-foreground">
          Redirecting to dashboard in {countdown} seconds...
        </p>
        <Link
          to="/"
          className="inline-flex items-center gap-2 rounded-lg bg-accent px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-accent/90"
        >
          <Home className="h-4 w-4" />
          Go to Dashboard
        </Link>
      </div>
    </PageContainer>
  );
}
