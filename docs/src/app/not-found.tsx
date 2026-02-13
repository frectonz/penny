import Link from "next/link";
import { PennyLogo } from "@/components/penny-logo";

export default function NotFound() {
  return (
    <div className="flex min-h-[60vh] flex-col items-center justify-center text-center">
      <PennyLogo size={64} color="#f97316" className="mb-6" />
      <h1 className="mb-2 text-4xl font-bold">404</h1>
      <p className="mb-6 text-fd-muted-foreground">Page not found</p>
      <Link
        href="/docs"
        className="inline-flex items-center rounded-lg bg-fd-primary px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-fd-primary/90"
      >
        Back to Docs
      </Link>
    </div>
  );
}
