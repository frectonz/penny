const AUTH_TOKEN_KEY = 'penny_auth_token';

export function getStoredAuth(): string | null {
  return localStorage.getItem(AUTH_TOKEN_KEY);
}

export function setStoredAuth(password: string): void {
  const encoded = btoa(password);
  localStorage.setItem(AUTH_TOKEN_KEY, encoded);
}

export function clearStoredAuth(): void {
  localStorage.removeItem(AUTH_TOKEN_KEY);
}
