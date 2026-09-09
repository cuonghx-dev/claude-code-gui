import { useQuery } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { claudeDirectoryTree } from '@/utils/ipc'

/** Global `~/.claude` tree, plus the project tree when a path is selected. */
export const useClaudeDirectoryTree = (projectPath?: MaybeRefOrGetter<string | undefined>) =>
  useQuery({
    queryKey: computed(() => qk.claudeDirectory.tree(toValue(projectPath))),
    queryFn: () => claudeDirectoryTree(toValue(projectPath) || undefined),
  })
