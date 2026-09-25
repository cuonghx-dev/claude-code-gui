import { parse } from 'yaml'

export type FrontmatterStatus =
  | { kind: 'none' }
  | { kind: 'valid' }
  | { kind: 'error'; message: string; line?: number }

/**
 * Validate the leading `---` YAML block of a markdown file. Line numbers in
 * the result are 1-based and relative to the whole file.
 */
export function checkFrontmatter(content: string): FrontmatterStatus {
  const lines = content.split('\n')
  if (lines[0]?.trim() !== '---') return { kind: 'none' }
  const end = lines.findIndex((l, i) => i > 0 && l.trim() === '---')
  if (end === -1) return { kind: 'error', message: 'Frontmatter is missing its closing ---', line: 1 }
  try {
    parse(lines.slice(1, end).join('\n'))
    return { kind: 'valid' }
  } catch (e) {
    const err = e as { message?: string; linePos?: Array<{ line: number }> }
    const line = err.linePos?.[0]?.line
    const message = (err.message ?? String(e)).split('\n')[0]
    return { kind: 'error', message, line: line !== undefined ? line + 1 : undefined }
  }
}
