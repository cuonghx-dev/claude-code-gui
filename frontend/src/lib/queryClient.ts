import { QueryClient } from '@tanstack/vue-query'

// Singleton; imported by main.ts (for VueQueryPlugin) and by fsListener.ts
// (for `queryClient.invalidateQueries`). ADR 0002.
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30_000,
      gcTime: 5 * 60_000,
      retry: 1,
      refetchOnWindowFocus: false,
    },
    mutations: {
      retry: 0,
    },
  },
})
