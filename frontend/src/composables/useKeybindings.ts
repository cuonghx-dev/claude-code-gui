import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { qk } from '@/lib/queryKeys'
import {
  keybindingsCreate,
  keybindingsGet,
  keybindingsPut,
  keybindingsRawGet,
  keybindingsRawPut,
  keybindingsValidate,
} from '@/utils/ipc'
import type { Keybinding } from '@/types/ipc'

export const useKeybindings = () =>
  useQuery({ queryKey: qk.keybindings.doc(), queryFn: keybindingsGet })

export const useKeybindingsRaw = () =>
  useQuery({ queryKey: qk.keybindings.raw(), queryFn: keybindingsRawGet })

export const useKeybindingsPut = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { bindings: Keybinding[]; expectedMtimeMs?: number }) =>
      keybindingsPut(v.bindings, v.expectedMtimeMs),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.keybindings.all }),
  })
}

export const useKeybindingsRawPut = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: (v: { content: string; expectedMtimeMs?: number }) =>
      keybindingsRawPut(v.content, v.expectedMtimeMs),
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.keybindings.all }),
  })
}

/** The file usually does not exist until something creates it. */
export const useKeybindingsCreate = () => {
  const qc = useQueryClient()
  return useMutation({
    mutationFn: keybindingsCreate,
    onSuccess: () => qc.invalidateQueries({ queryKey: qk.keybindings.all }),
  })
}

export const useValidateChord = () =>
  useMutation({ mutationFn: (chord: string) => keybindingsValidate(chord) })
