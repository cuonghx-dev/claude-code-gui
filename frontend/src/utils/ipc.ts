import { invoke } from '@tauri-apps/api/core'
import type {
  Agent,
  ClaudeCliInfo,
  Command,
  FileNode,
  McpScope,
  McpServer,
  Message,
  OutputStyle,
  OutputStyleScope,
  Page,
  Plan,
  Plugin,
  PluginDetail,
  Project,
  ProjectInfo,
  SessionSummary,
  Settings,
  Skill,
} from '@/types/ipc'

export { invoke } from '@tauri-apps/api/core'
export { listen } from '@tauri-apps/api/event'

// Agents
export const agentsList = () => invoke<Agent[]>('agents_list')
export const agentsGet = (slug: string) => invoke<Agent>('agents_get', { slug })
export const agentsSkillCounts = () => invoke<Record<string, number>>('agents_skill_counts')

// Commands
export const commandsList = () => invoke<Command[]>('commands_list')
export const commandsGet = (slug: string) => invoke<Command>('commands_get', { slug })

// Skills
export const skillsList = () => invoke<Skill[]>('skills_list')
export const skillsGet = (slug: string) => invoke<Skill>('skills_get', { slug })

// Plans
export const plansList = () => invoke<Plan[]>('plans_list')
export const plansGet = (slug: string) => invoke<Plan>('plans_get', { slug })

// Output styles
export const outputStylesList = (workingDir?: string) =>
  invoke<OutputStyle[]>('output_styles_list', { workingDir })
export const outputStylesGet = (id: string, scope: OutputStyleScope, workingDir?: string) =>
  invoke<OutputStyle>('output_styles_get', { id, scope, workingDir })

// MCP
export const mcpList = (scope: McpScope, workingDir?: string) =>
  invoke<McpServer[]>('mcp_list', { scope, workingDir })
export const mcpGet = (name: string, scope: McpScope, workingDir?: string) =>
  invoke<McpServer>('mcp_get', { name, scope, workingDir })

// Plugins
export const pluginsList = () => invoke<Plugin[]>('plugins_list')
export const pluginsGet = (id: string) => invoke<PluginDetail>('plugins_get', { id })

// Projects
export const projectsList = () => invoke<Project[]>('projects_list')
export const projectsGet = (name: string) => invoke<Project>('projects_get', { name })
export const projectsResolve = (path: string) => invoke<ProjectInfo>('projects_resolve', { path })
export const projectsFiles = (name: string, subPath?: string) =>
  invoke<FileNode[]>('projects_files', { name, subPath })

// Sessions
export const sessionsListForProject = (name: string) =>
  invoke<SessionSummary[]>('sessions_list_for_project', { name })
export const sessionsMessages = (
  projectName: string,
  sessionId: string,
  afterIndex?: number,
  limit?: number,
) =>
  invoke<Page<Message>>('sessions_messages', {
    projectName,
    sessionId,
    afterIndex,
    limit,
  })

// Settings / debug
export const settingsGet = () => invoke<Settings>('settings_get')
export const debugClaudeCli = () => invoke<ClaudeCliInfo | null>('debug_claude_cli')
export const appVersion = () => invoke<string>('app_version')
