// Typed `invoke()` wrapper layer. Phase 1+ populates per-domain invokers
// here that wrap `invoke()` with the right input/output types from
// `frontend/src/types/ipc/`.

export { invoke } from '@tauri-apps/api/core'
export { listen } from '@tauri-apps/api/event'
