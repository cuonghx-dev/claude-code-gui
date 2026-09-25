/**
 * Claude Code agents declare `color:` as a name (`orange`, `blue`, …). Map
 * those onto the design's muted palette; anything else (a hex, rgb()) is
 * passed through as-is.
 */
const NAMED: Record<string, string> = {
  red: '#B03A2E',
  orange: '#C2552D',
  yellow: '#C99A2E',
  amber: '#C99A2E',
  green: '#3F7A4E',
  emerald: '#3F7A4E',
  teal: '#2E7F7A',
  cyan: '#2E7F95',
  blue: '#2F5E9E',
  indigo: '#4B55A0',
  purple: '#7A4FA0',
  violet: '#7A4FA0',
  pink: '#B2507F',
  magenta: '#A0428F',
  gray: '#8A867C',
  grey: '#8A867C',
  black: '#1F1E1B',
  white: '#DEDAD1',
}

export function agentColor(color: string | null | undefined): string {
  if (!color) return '#C2BDB2'
  return NAMED[color.trim().toLowerCase()] ?? color
}
