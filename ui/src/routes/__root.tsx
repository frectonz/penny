import { TanStackDevtools } from '@tanstack/react-devtools';
import type { QueryClient } from '@tanstack/react-query';
import { createRootRouteWithContext, Outlet } from '@tanstack/react-router';
import { TanStackRouterDevtoolsPanel } from '@tanstack/react-router-devtools';
import Header from '../components/Header';
import LoginPage from '../components/LoginPage';
import { AuthProvider, useAuth } from '../contexts/AuthContext';
import TanStackQueryDevtools from '../integrations/tanstack-query/devtools';

interface MyRouterContext {
  queryClient: QueryClient;
}

function RootContent() {
  const { isAuthRequired, isAuthenticated, isLoading } = useAuth();

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-background">
        <div className="text-muted-foreground">Loading...</div>
      </div>
    );
  }

  if (isAuthRequired && !isAuthenticated) {
    return <LoginPage />;
  }

  return (
    <>
      <Header />
      <Outlet />
      <TanStackDevtools
        config={{
          position: 'bottom-right',
        }}
        plugins={[
          {
            name: 'Tanstack Router',
            render: <TanStackRouterDevtoolsPanel />,
          },
          TanStackQueryDevtools,
        ]}
      />
    </>
  );
}

export const Route = createRootRouteWithContext<MyRouterContext>()({
  component: () => (
    <AuthProvider>
      <RootContent />
    </AuthProvider>
  ),
});
