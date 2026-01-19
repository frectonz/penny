import { useQuery } from '@tanstack/react-query';
import { createFileRoute } from '@tanstack/react-router';
import { $fetch } from '../lib/api';

export const Route = createFileRoute('/')({
  component: App,
});

function App() {
  const { data, isLoading, error } = useQuery({
    queryKey: ['version'],
    queryFn: async () => {
      const { data, error } = await $fetch('/api/version');
      if (error) throw error;
      return data;
    },
  });

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-[#282c34] text-white">
      <h1 className="text-4xl font-bold mb-4">Penny</h1>
      {isLoading && <p className="text-gray-400">Loading...</p>}
      {error && <p className="text-red-400">Error: {error.message}</p>}
      {data && (
        <p className="text-[#61dafb] text-xl">Version: {data.version}</p>
      )}
    </div>
  );
}
