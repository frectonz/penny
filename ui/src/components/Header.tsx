import { Link } from '@tanstack/react-router';

export default function Header() {
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
      </nav>
    </header>
  );
}
