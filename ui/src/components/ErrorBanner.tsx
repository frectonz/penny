import { AlertTriangle } from 'lucide-react';
import { cn } from '@/lib/utils';

export function ErrorBanner({
  message,
  className,
}: {
  message: string;
  className?: string;
}) {
  return (
    <div
      className={cn(
        'px-5 py-4 mb-8 border border-destructive rounded bg-destructive/10 flex items-center gap-3',
        className,
      )}
    >
      <AlertTriangle className="w-5 h-5 text-destructive shrink-0" />
      <p className="text-destructive text-sm">{message}</p>
    </div>
  );
}
