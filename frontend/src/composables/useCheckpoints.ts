import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type MaybeRefOrGetter, toValue } from 'vue'
import { qk } from '@/lib/queryKeys'
import { fileHistoryCheckpoints, fileHistoryDiff, fileHistoryRestore } from '@/utils/ipc'
import type { DiffSide } from '@/types/ipc'

export const useCheckpoints = (
  projectName: MaybeRefOrGetter<string>,
  sessionId: MaybeRefOrGetter<string>,
) =>
  useQuery({
    queryKey: computed(() => qk.checkpoints.list(toValue(sessionId))),
    queryFn: () => fileHistoryCheckpoints(toValue(projectName), toValue(sessionId)),
    enabled: computed(() => !!toValue(sessionId) && !!toValue(projectName)),
  })

export const useCheckpointDiff = (
  sessionId: MaybeRefOrGetter<string>,
  left: MaybeRefOrGetter<DiffSide | null>,
  right: MaybeRefOrGetter<DiffSide | null>,
) =>
  useQuery({
    queryKey: computed(() =>
      qk.checkpoints.diff(
        toValue(sessionId),
        JSON.stringify(toValue(left)),
        JSON.stringify(toValue(right)),
      ),
    ),
    queryFn: () => fileHistoryDiff(toValue(sessionId), toValue(left)!, toValue(right)!),
    enabled: computed(() => !!toValue(left) && !!toValue(right)),
    // Blobs are immutable; only the working-tree side can change, and the
    // watcher invalidates that.
    staleTime: Infinity,
  })

export const useCheckpointRestore = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: {
      projectName: string
      sessionId: string
      backupFileName: string
      dest: string
    }) => fileHistoryRestore(v.projectName, v.sessionId, v.backupFileName, v.dest),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.checkpoints.all }),
  })
}
