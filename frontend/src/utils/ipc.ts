import { invoke } from '@tauri-apps/api/core'
import type {
  Agent,
  AgentImport,
  AgentInput,
  AppConfig,
  AvailablePlugin,
  ClaudeCliInfo,
  Command,
  CommandInput,
  DirEntry,
  FileNode,
  GitStatus,
  ImproveRequest,
  MarketplaceSource,
  MarketplaceSourceInput,
  McpCapabilities,
  McpImportPayload,
  McpScope,
  McpServer,
  McpServerInput,
  Message,
  OutputStyle,
  OutputStyleInput,
  OutputStyleScope,
  Page,
  Plan,
  PlanInput,
  Plugin,
  PluginDetail,
  Project,
  ProjectInfo,
  RelationshipsGraph,
  RequestId,
  SessionId,
  SessionSummary,
  SetupPayload,
  Settings,
  Skill,
  SkillImportSource,
  SkillInput,
  TerminalOpts,
  TerminalSession,
} from '@/types/ipc'

export { invoke } from '@tauri-apps/api/core'
export { listen } from '@tauri-apps/api/event'

// Agents
export const agentsList = () => invoke<Agent[]>('agents_list')
export const agentsGet = (slug: string) => invoke<Agent>('agents_get', { slug })
export const agentsSkillCounts = () => invoke<Record<string, number>>('agents_skill_counts')
export const agentsCreate = (input: AgentInput) => invoke<Agent>('agents_create', { input })
export const agentsUpdate = (slug: string, input: AgentInput) =>
  invoke<Agent>('agents_update', { slug, input })
export const agentsDelete = (slug: string) => invoke<void>('agents_delete', { slug })
export const agentsExport = (slug: string) => invoke<string>('agents_export', { slug })
export const agentsImport = (payload: AgentImport) => invoke<Agent>('agents_import', { payload })
export const agentsImproveInstructions = (input: ImproveRequest) =>
  invoke<RequestId>('agents_improve_instructions', { input })

// Commands
export const commandsList = () => invoke<Command[]>('commands_list')
export const commandsGet = (slug: string) => invoke<Command>('commands_get', { slug })
export const commandsCreate = (input: CommandInput) =>
  invoke<Command>('commands_create', { input })
export const commandsUpdate = (slug: string, input: CommandInput) =>
  invoke<Command>('commands_update', { slug, input })
export const commandsDelete = (slug: string) => invoke<void>('commands_delete', { slug })

// Skills
export const skillsList = () => invoke<Skill[]>('skills_list')
export const skillsGet = (slug: string) => invoke<Skill>('skills_get', { slug })
export const skillsCreate = (input: SkillInput) => invoke<Skill>('skills_create', { input })
export const skillsUpdate = (slug: string, input: SkillInput) =>
  invoke<Skill>('skills_update', { slug, input })
export const skillsDelete = (slug: string) => invoke<void>('skills_delete', { slug })
export const skillsExport = (slug: string) => invoke<number[]>('skills_export', { slug })
export const skillsImport = (source: SkillImportSource) =>
  invoke<Skill[]>('skills_import', { source })

// Plans
export const plansList = () => invoke<Plan[]>('plans_list')
export const plansGet = (slug: string) => invoke<Plan>('plans_get', { slug })
export const plansCreate = (input: PlanInput) => invoke<Plan>('plans_create', { input })
export const plansUpdate = (slug: string, input: PlanInput) =>
  invoke<Plan>('plans_update', { slug, input })
export const plansDelete = (slug: string) => invoke<void>('plans_delete', { slug })

// Output styles
export const outputStylesList = (workingDir?: string) =>
  invoke<OutputStyle[]>('output_styles_list', { workingDir })
export const outputStylesGet = (id: string, scope: OutputStyleScope, workingDir?: string) =>
  invoke<OutputStyle>('output_styles_get', { id, scope, workingDir })
export const outputStylesCreate = (input: OutputStyleInput) =>
  invoke<OutputStyle>('output_styles_create', { input })
export const outputStylesDelete = (id: string, scope: OutputStyleScope, workingDir?: string) =>
  invoke<void>('output_styles_delete', { id, scope, workingDir })

// MCP
export const mcpList = (scope: McpScope, workingDir?: string) =>
  invoke<McpServer[]>('mcp_list', { scope, workingDir })
export const mcpGet = (name: string, scope: McpScope, workingDir?: string) =>
  invoke<McpServer>('mcp_get', { name, scope, workingDir })
export const mcpCreate = (input: McpServerInput, scope: McpScope, workingDir?: string) =>
  invoke<McpServer>('mcp_create', { input, scope, workingDir })
export const mcpDelete = (name: string, scope: McpScope, workingDir?: string) =>
  invoke<void>('mcp_delete', { name, scope, workingDir })
export const mcpImport = (payload: McpImportPayload) =>
  invoke<McpServer[]>('mcp_import', { payload })
export const mcpCapabilities = (
  name: string,
  scope: McpScope,
  workingDir?: string,
) => invoke<McpCapabilities>('mcp_capabilities', { name, scope, workingDir })

// Relationships
export const relationshipsGraph = () =>
  invoke<RelationshipsGraph>('relationships_graph')

// Plugins
export const pluginsList = () => invoke<Plugin[]>('plugins_list')
export const pluginsGet = (id: string) => invoke<PluginDetail>('plugins_get', { id })
export const pluginsDelete = (id: string) => invoke<void>('plugins_delete', { id })
export const pluginsSetEnabled = (id: string, enabled: boolean) =>
  invoke<void>('plugins_set_enabled', { id, enabled })
export const pluginsUpdateSkills = (id: string, slugs: string[]) =>
  invoke<void>('plugins_update_skills', { id, slugs })

// Marketplace
export const marketplaceAvailable = () =>
  invoke<AvailablePlugin[]>('marketplace_available')
export const marketplaceSourcesList = () =>
  invoke<MarketplaceSource[]>('marketplace_sources_list')
export const marketplaceSourcesAdd = (input: MarketplaceSourceInput) =>
  invoke<void>('marketplace_sources_add', { input })
export const marketplaceSourcesRemove = (name: string) =>
  invoke<void>('marketplace_sources_remove', { name })
export const marketplaceSourcesUpdate = (name: string) =>
  invoke<void>('marketplace_sources_update', { name })
export const marketplaceInstall = (name: string, source: string) =>
  invoke<RequestId>('marketplace_install', { name, source })
export const marketplaceUninstall = (id: string) =>
  invoke<void>('marketplace_uninstall', { id })

// Watcher
export const watchProjectDir = (path: string) =>
  invoke<string>('watch_project_dir', { path })
export const unwatchPath = (id: string) => invoke<void>('unwatch_path', { id })

// Terminal
export const terminalSessionCreate = (opts: TerminalOpts) =>
  invoke<SessionId>('terminal_session_create', { opts })
export const terminalSessionInput = (sessionId: string, data: string) =>
  invoke<void>('terminal_session_input', { sessionId, data })
export const terminalSessionResize = (sessionId: string, cols: number, rows: number) =>
  invoke<void>('terminal_session_resize', { sessionId, cols, rows })
export const terminalSessionKill = (sessionId: string) =>
  invoke<void>('terminal_session_kill', { sessionId })
export const terminalSessionsList = () =>
  invoke<TerminalSession[]>('terminal_sessions_list')
export const terminalSessionGet = (sessionId: string) =>
  invoke<TerminalSession>('terminal_session_get', { sessionId })
export const commandsExecute = (
  slug: string,
  args?: string,
  workingDir?: string,
) => invoke<SessionId>('commands_execute', { slug, args, workingDir })

// Projects
export const projectsList = () => invoke<Project[]>('projects_list')
export const projectsGet = (name: string) => invoke<Project>('projects_get', { name })
export const projectsResolve = (path: string) => invoke<ProjectInfo>('projects_resolve', { path })
export const projectsFiles = (name: string, subPath?: string) =>
  invoke<FileNode[]>('projects_files', { name, subPath })
export const projectsCreate = (path: string) => invoke<Project>('projects_create', { path })
export const projectsRename = (name: string, newName: string) =>
  invoke<void>('projects_rename', { name, newName })
export const projectsDelete = (name: string) => invoke<void>('projects_delete', { name })
export const projectsGitStatus = (name: string) =>
  invoke<GitStatus>('projects_git_status', { name })
export const projectsSettingsGet = (name: string) =>
  invoke<Settings>('projects_settings_get', { name })
export const projectsSettingsPut = (name: string, settings: Settings) =>
  invoke<void>('projects_settings_put', { name, settings })
export const projectsClaudeMdGet = (name: string) =>
  invoke<string>('projects_claude_md_get', { name })
export const projectsClaudeMdPut = (name: string, content: string) =>
  invoke<void>('projects_claude_md_put', { name, content })

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

// Settings / config / setup
export const settingsGet = () => invoke<Settings>('settings_get')
export const settingsPut = (settings: Settings) => invoke<void>('settings_put', { settings })
export const configGet = () => invoke<AppConfig>('config_get')
export const configSet = (config: AppConfig) => invoke<void>('config_set', { config })
export const setupFinalize = (payload: SetupPayload) =>
  invoke<void>('setup_finalize', { payload })

// Filesystem utilities
export const directoriesList = (parent: string) =>
  invoke<DirEntry[]>('directories_list', { parent })
export const filesRead = (path: string) => invoke<string>('files_read', { path })
export const fsHomeDir = () => invoke<string>('fs_home_dir')

// Debug
export const debugClaudeCli = () => invoke<ClaudeCliInfo | null>('debug_claude_cli')
export const appVersion = () => invoke<string>('app_version')
