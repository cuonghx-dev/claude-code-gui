import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { useRoute } from 'vue-router'
import { useProject } from '@/composables/useProjects'
import { qk } from '@/lib/queryKeys'
import type { McpImportPayload, McpScope, McpServerInput } from '@/types/ipc'
import {
  mcpCapabilities,
  mcpCreate,
  mcpDelete,
  mcpGet,
  mcpImport,
  mcpList,
} from '@/utils/ipc'

export const useMcpList = (
  scope: MaybeRefOrGetter<McpScope>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true,
) =>
  useQuery({
    queryKey: computed(() => qk.mcp.list(toValue(scope), toValue(workingDir))),
    queryFn: () => mcpList(toValue(scope), toValue(workingDir)),
    enabled: computed(() => toValue(enabled)),
  })

export const useMcpServer = (
  name: MaybeRefOrGetter<string>,
  scope: MaybeRefOrGetter<McpScope>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
  enabled: MaybeRefOrGetter<boolean> = true,
) =>
  useQuery({
    queryKey: computed(() => qk.mcp.get(toValue(name), toValue(scope), toValue(workingDir))),
    queryFn: () => mcpGet(toValue(name), toValue(scope), toValue(workingDir)),
    enabled: computed(() => !!toValue(name) && toValue(enabled)),
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

export const useMcpCapabilities = (
  name: MaybeRefOrGetter<string>,
  scope: MaybeRefOrGetter<McpScope>,
  workingDir?: MaybeRefOrGetter<string | undefined>,
  enabled?: MaybeRefOrGetter<boolean>,
) =>
  useQuery({
    queryKey: computed(() =>
      qk.mcp.capabilities(toValue(name), toValue(scope), toValue(workingDir)),
    ),
    queryFn: () => mcpCapabilities(toValue(name), toValue(scope), toValue(workingDir)),
    // The probe is a 5s spawn; only fire when the caller flips enabled true.
    enabled: computed(() => !!toValue(name) && !!toValue(enabled)),
    // Capability lists are stable enough that we can avoid refetching
    // every focus.
    staleTime: 60_000,
  })

export const useMcpImport = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (payload: McpImportPayload) => mcpImport(payload),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.mcp.all }),
  })
}

/**
 * The MCP scope the pages act on, carried in the route query
 * (`?scope=project&project=<name>`) so list → detail → back keeps it.
 * Project scope reads/writes `<workingDir>/.mcp.json`; global is `~/.claude.json`.
 */
export const useMcpScope = () => {
  const route = useRoute()
  const scope = computed<McpScope>(() => (route.query.scope === 'project' ? 'project' : 'global'))
  const projectName = computed(() =>
    typeof route.query.project === 'string' ? route.query.project : '',
  )
  const project = useProject(() => (scope.value === 'project' ? projectName.value : ''))
  const workingDir = computed(() =>
    scope.value === 'project' ? project.data.value?.workingDir ?? undefined : undefined,
  )
  /** Project scope is usable only once its working dir has resolved. */
  const ready = computed(() => scope.value === 'global' || !!workingDir.value)
  const query = computed(() =>
    scope.value === 'project' ? { scope: 'project', project: projectName.value } : {},
  )
  const fileLabel = computed(() =>
    scope.value === 'project'
      ? `${workingDir.value ?? '<project>'}/.mcp.json`
      : '~/.claude.json',
  )
  return { scope, projectName, workingDir, ready, query, fileLabel }
}
