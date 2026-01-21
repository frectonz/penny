import type { LucideIcon } from 'lucide-react';
import { Skeleton } from '@/components/ui/skeleton';

export function StatCard({
  title,
  value,
  icon: Icon,
  iconColor,
  valueClassName,
}: {
  title: string;
  value: string | number;
  icon: LucideIcon;
  iconColor: string;
  valueClassName?: string;
}) {
  return (
    <div className="p-5 border border-border rounded bg-card h-full">
      <div className="flex items-center gap-2 mb-3">
        <Icon className={`w-4 h-4 ${iconColor} shrink-0`} />
        <span className="text-xs uppercase tracking-widest text-muted-foreground">
          {title}
        </span>
      </div>
      <p
        className={`text-3xl font-semibold tabular-nums ${valueClassName ?? 'text-foreground'}`}
      >
        {value}
      </p>
    </div>
  );
}

export function StatCardSkeleton() {
  return (
    <div className="p-5 border border-border rounded bg-card h-full">
      <div className="flex items-center gap-2 mb-3">
        <Skeleton className="h-4 w-4 rounded shrink-0" />
        <Skeleton className="h-4 w-24" />
      </div>
      <Skeleton className="h-9 w-20" />
    </div>
  );
}
