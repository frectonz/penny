import { useState } from 'react';
import { AlertTriangle, ChevronDown } from 'lucide-react';
import { cn } from '@/lib/utils';

export function ErrorBanner({
  message,
  details,
  className,
}: {
  message: string;
  details?: string;
  className?: string;
}) {
  const [expanded, setExpanded] = useState(false);
  const hasDetails = !!details;

  return (
    <div
      className={cn(
        'px-5 py-4 mb-8 border border-destructive rounded bg-destructive/10',
        className,
      )}
    >
      <div className="flex items-start gap-3">
        <AlertTriangle className="w-5 h-5 text-destructive shrink-0 mt-0.5" />
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between gap-2">
            <p className="text-destructive text-sm">{message}</p>
            {hasDetails && (
              <button
                type="button"
                onClick={() => setExpanded(!expanded)}
                className="shrink-0 p-1 rounded hover:bg-destructive/20 transition-colors"
              >
                <ChevronDown
                  className={cn(
                    'w-4 h-4 text-destructive transition-transform',
                    expanded && 'rotate-180',
                  )}
                />
              </button>
            )}
          </div>
          {hasDetails && expanded && (
            <pre className="mt-3 pt-3 border-t border-destructive/30 text-destructive/80 text-xs font-mono whitespace-pre-wrap break-all">
              {details}
            </pre>
          )}
        </div>
      </div>
    </div>
  );
}
