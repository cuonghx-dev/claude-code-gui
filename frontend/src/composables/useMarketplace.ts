import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { qk } from '@/lib/queryKeys'
import type { MarketplaceSourceInput } from '@/types/ipc'
import {
  marketplaceAvailable,
  marketplaceInstall,
  marketplaceSourcesAdd,
  marketplaceSourcesList,
  marketplaceSourcesRemove,
  marketplaceSourcesUpdate,
  marketplaceUninstall,
} from '@/utils/ipc'

export const useMarketplaceAvailable = () =>
  useQuery({ queryKey: qk.marketplace.available(), queryFn: marketplaceAvailable })

export const useMarketplaceSources = () =>
  useQuery({ queryKey: qk.marketplace.sources(), queryFn: marketplaceSourcesList })

export const useMarketplaceSourceAdd = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (input: MarketplaceSourceInput) => marketplaceSourcesAdd(input),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.marketplace.sources() }),
  })
}

export const useMarketplaceSourceRemove = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (name: string) => marketplaceSourcesRemove(name),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.marketplace.sources() })
      qc.invalidateQueries({ queryKey: qk.marketplace.available() })
    },
  })
}

export const useMarketplaceSourceUpdate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (name: string) => marketplaceSourcesUpdate(name),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: qk.marketplace.sources() })
      qc.invalidateQueries({ queryKey: qk.marketplace.available() })
    },
  })
}

/**
 * Returns the request id immediately. The caller is responsible for
 * wiring `useAsyncRequest('marketplace:install', requestId)`.
 */
export const useMarketplaceInstall = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({ name, source }: { name: string; source: string }) =>
      marketplaceInstall(name, source),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.plugins.all }),
  })
}

export const useMarketplaceUninstall = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (id: string) => marketplaceUninstall(id),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.plugins.all }),
  })
}
