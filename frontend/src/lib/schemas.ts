import { z } from 'zod'
// Schemas are intentionally `Partial`-ish: the IPC types use `T | null` for
// optional frontmatter fields (ts-rs renders `Option<T>` as `T | null`),
// while zod prefers `T | undefined`. Forms convert empty strings → null on
// submit, so the runtime shape is correct even though the static `satisfies`
// constraint can't bridge the gap. We keep the schemas free of `satisfies`
// and let the form components cast the produced object to the IPC type.

const SLUG = z
  .string()
  .min(1, 'required')
  .max(64, 'max 64 chars')
  .regex(/^[a-z0-9_]+(?:-[a-z0-9_]+)*$/, 'lowercase letters, digits, "-" or "_"')

const REL_DIR = z
  .string()
  .regex(
    /^$|^[A-Za-z0-9_.][A-Za-z0-9_./-]*$/,
    'relative path; no leading "/" or ".."',
  )
  .refine((s) => !s.includes('..'), 'must not contain ".."')

export const agentSchema = z.object({
  slug: SLUG,
  directory: REL_DIR.default(''),
  frontmatter: z
    .object({
      name: z.string().optional(),
      description: z.string().optional(),
      model: z.enum(['opus', 'sonnet', 'haiku']).nullish(),
      color: z
        .string()
        .regex(/^#[0-9a-fA-F]{6}$/, 'hex color, e.g. #7c3aed')
        .optional()
        .or(z.literal('')),
      memory: z.enum(['user', 'project', 'local', 'none']).nullish(),
      skills: z.array(z.string()).default([]),
      tools: z.array(z.string()).default([]),
      extra: z.record(z.unknown()).default({}),
    })
    .passthrough(),
  body: z.string(),
})

export const commandSchema = z.object({
  slug: SLUG,
  directory: REL_DIR.default(''),
  frontmatter: z
    .object({
      name: z.string().optional(),
      description: z.string().optional(),
      argumentHint: z.string().optional(),
      allowedTools: z.array(z.string()).default([]),
      agent: z.string().optional(),
      extra: z.record(z.unknown()).default({}),
    })
    .passthrough(),
  body: z.string(),
})

export const skillSchema = z.object({
  slug: SLUG,
  frontmatter: z
    .object({
      name: z.string().optional(),
      description: z.string().optional(),
      context: z.enum(['when', 'always']).nullish(),
      agent: z.string().optional(),
      extra: z.record(z.unknown()).default({}),
    })
    .passthrough(),
  body: z.string(),
})

export const planSchema = z.object({
  slug: SLUG,
  body: z.string(),
})

export const outputStyleSchema = z.object({
  id: SLUG,
  scope: z.enum(['global', 'project']),
  workingDir: z.string().optional(),
  frontmatter: z
    .object({
      name: z.string().optional(),
      description: z.string().optional(),
      keepCodingInstructions: z.boolean().nullish(),
      extra: z.record(z.unknown()).default({}),
    })
    .passthrough(),
  body: z.string(),
})

const mcpStdio = z.object({
  kind: z.literal('stdio'),
  command: z.string().min(1, 'required'),
  args: z.array(z.string()).default([]),
  env: z.record(z.string()).default({}),
})

const mcpHttp = z.object({
  kind: z.literal('httpSse'),
  url: z.string().url('must be a valid URL'),
  headers: z.record(z.string()).default({}),
})

export const mcpServerSchema = z.object({
  name: z
    .string()
    .min(1, 'required')
    .max(64, 'max 64 chars')
    .regex(/^[A-Za-z0-9][A-Za-z0-9._-]*$/, 'letters, digits, ".", "-", "_"'),
  transport: z.discriminatedUnion('kind', [mcpStdio, mcpHttp]),
})

export type AgentForm = z.infer<typeof agentSchema>
export type CommandForm = z.infer<typeof commandSchema>
export type SkillForm = z.infer<typeof skillSchema>
export type PlanForm = z.infer<typeof planSchema>
export type OutputStyleForm = z.infer<typeof outputStyleSchema>
export type McpServerForm = z.infer<typeof mcpServerSchema>

/** Map a zod error to `{ "path.to.field": "message" }`. Useful for FormField. */
export function flattenErrors(err: z.ZodError): Record<string, string> {
  const out: Record<string, string> = {}
  for (const issue of err.issues) {
    const key = issue.path.map(String).join('.')
    if (!out[key]) out[key] = issue.message
  }
  return out
}
