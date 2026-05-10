import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import type { McpImportPayload, McpScope, McpServerInput } from '@/types/ipc'
import { mcpCreate, mcpDelete, mcpGet, mcpImport, mcpList } from '@/utils/ipc'

export const useMcpList = (
  scope: MaybeRefOrGetter<McpScope>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
) =>
  useQuery({
    queryKey: computed(() => qk.mcp.list(toValue(scope), toValue(workingDir))),
    queryFn: () => mcpList(toValue(scope), toValue(workingDir)),
  })

export const useMcpServer = (
  name: MaybeRefOrGetter<string>,
  scope: MaybeRefOrGetter<McpScope>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
) =>
  useQuery({
    queryKey: computed(() => qk.mcp.get(toValue(name), toValue(scope), toValue(workingDir))),
    queryFn: () => mcpGet(toValue(name), toValue(scope), toValue(workingDir)),
    enabled: computed(() => !!toValue(name)),
  })

export const useMcpCreate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({
      input,
      scope,
      workingDir,
    }: {
      input: McpServerInput
      scope: McpScope
      workingDir?: string
    }) => mcpCreate(input, scope, workingDir),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.mcp.all }),
  })
}

export const useMcpDelete = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: ({
      name,
      scope,
      workingDir,
    }: {
      name: string
      scope: McpScope
      workingDir?: string
    }) => mcpDelete(name, scope, workingDir),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.mcp.all }),
  })
}

export const useMcpImport = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (payload: McpImportPayload) => mcpImport(payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.mcp.all }),
  })
}
