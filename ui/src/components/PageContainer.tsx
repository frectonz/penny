import type { ReactNode } from 'react';
import { cn } from '@/lib/utils';

export function PageContainer({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <div
      className={cn(
        'min-h-[calc(100vh-57px)] bg-background bg-grid p-8',
        className,
      )}
    >
      {children}
    </div>
  );
}
