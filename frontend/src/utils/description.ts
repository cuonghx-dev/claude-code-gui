const MAX_LEN = 200

export function describe(
  description: string | null | undefined,
  body?: string | null,
): string {
  const fm = description?.trim()
  if (fm) return fm
  const fallback = firstMeaningfulLine(body)
  return fallback ?? '—'
}

export function describePlugin(
  description: string | null | undefined,
  skills: ReadonlyArray<string>,
): string {
  const fm = description?.trim()
  if (fm) return fm
  if (skills.length === 0) return '—'
  const head = skills.slice(0, 3).join(', ')
  const more = skills.length > 3 ? `, +${skills.length - 3} more` : ''
  return `Provides: ${head}${more}`
}

function firstMeaningfulLine(body: string | null | undefined): string | null {
  if (!body) return null
  for (const raw of body.split(/\r?\n/)) {
    const line = raw.trim()
    if (!line) continue
    if (line.startsWith('#')) continue
    if (line.startsWith('---')) continue
    if (line.startsWith('```')) continue
    return line.length > MAX_LEN ? line.slice(0, MAX_LEN - 1).trimEnd() + '…' : line
  }
  return null
}
