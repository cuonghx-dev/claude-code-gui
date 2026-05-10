import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import type { McpScope } from '@/types/ipc'
import { mcpGet, mcpList } from '@/utils/ipc'

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
