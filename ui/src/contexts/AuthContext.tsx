import { useQuery, useQueryClient } from '@tanstack/react-query';
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useState,
} from 'react';
import { $fetch } from '@/lib/api';
import { clearStoredAuth, getStoredAuth, setStoredAuth } from '@/lib/auth';

interface AuthContextType {
  isAuthRequired: boolean;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (password: string) => Promise<boolean>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const queryClient = useQueryClient();
  const [isAuthenticated, setIsAuthenticated] = useState(
    () => !!getStoredAuth(),
  );

  const { data: authStatus, isLoading } = useQuery({
    queryKey: ['auth-status'],
    queryFn: () => $fetch('/api/auth/status'),
    staleTime: Number.POSITIVE_INFINITY,
  });

  const isAuthRequired = authStatus?.auth_required ?? false;

  // Verify stored credentials on mount when auth is required
  useEffect(() => {
    if (!isAuthRequired || !getStoredAuth()) {
      return;
    }

    // Test if stored credentials are valid
    $fetch('/api/version')
      .then(() => setIsAuthenticated(true))
      .catch(() => {
        clearStoredAuth();
        setIsAuthenticated(false);
      });
  }, [isAuthRequired]);

  const login = useCallback(
    async (password: string): Promise<boolean> => {
      setStoredAuth(password);

      try {
        await $fetch('/api/version');
        setIsAuthenticated(true);
        queryClient.invalidateQueries();
        return true;
      } catch {
        clearStoredAuth();
        setIsAuthenticated(false);
        return false;
      }
    },
    [queryClient],
  );

  const logout = useCallback(() => {
    clearStoredAuth();
    setIsAuthenticated(false);
    queryClient.invalidateQueries();
  }, [queryClient]);

  return (
    <AuthContext.Provider
      value={{
        isAuthRequired,
        isAuthenticated,
        isLoading,
        login,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
