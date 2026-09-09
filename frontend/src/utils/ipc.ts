import { invoke } from '@tauri-apps/api/core'
import type {
  Agent,
  AgentImport,
  AgentInput,
  AppConfig,
  AvailablePlugin,
  Checkpoint,
  ChordValidation,
  ClaudeCliInfo,
  EffectiveEntry,
  EffectivePermissions,
  CliHistoryDetail,
  CliHistoryEntry,
  ClaudeDirTree,
  Command,
  CommandInput,
  DirEntry,
  FileNode,
  GitStatus,
  HookGroup,
  Job,
  JobDetail,
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
  ActivityDay,
  IndexStats,
  DiffResult,
  DiffSide,
  HookInput,
  Keybinding,
  KeybindingsDoc,
  MemoryDoc,
  MemoryFile,
  MemoryPreview,
  Permissions,
  RawDoc,
  RuleIssue,
  ScopeInfo,
  SettingsScope,
  StatusLine,
  StatusLinePreview,
  Team,
  Thread,
  UsageQuery,
  UsageReport,
  TerminalOpts,
  TerminalSession,
  Workflow,
  WorkflowInput,
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
export const agentsUpdateRaw = (slug: string, content: string) =>
  invoke<Agent>('agents_update_raw', { slug, content })
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
export const commandsUpdateRaw = (slug: string, content: string) =>
  invoke<Command>('commands_update_raw', { slug, content })
export const commandsImportRaw = (slug: string, directory: string, content: string) =>
  invoke<Command>('commands_import_raw', { slug, directory, content })
export const commandsExport = (slug: string) => invoke<string>('commands_export', { slug })
export const commandsDelete = (slug: string) => invoke<void>('commands_delete', { slug })

// Skills
export const skillsList = () => invoke<Skill[]>('skills_list')
export const skillsGet = (slug: string) => invoke<Skill>('skills_get', { slug })
export const skillsCreate = (input: SkillInput) => invoke<Skill>('skills_create', { input })
export const skillsCreateRaw = (slug: string, content: string) =>
  invoke<Skill>('skills_create_raw', { slug, content })
export const skillsUpdate = (slug: string, input: SkillInput) =>
  invoke<Skill>('skills_update', { slug, input })
export const skillsUpdateRaw = (slug: string, content: string) =>
  invoke<Skill>('skills_update_raw', { slug, content })
export const skillsReadRaw = (slug: string) => invoke<string>('skills_read_raw', { slug })
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

// Workflows
export const workflowsList = () => invoke<Workflow[]>('workflows_list')
export const workflowsGet = (slug: string) => invoke<Workflow>('workflows_get', { slug })
export const workflowsCreate = (input: WorkflowInput) =>
  invoke<Workflow>('workflows_create', { input })
export const workflowsUpdate = (slug: string, input: WorkflowInput) =>
  invoke<Workflow>('workflows_update', { slug, input })
export const workflowsDelete = (slug: string) => invoke<void>('workflows_delete', { slug })

// Terminals (cli-history)
export const cliHistoryList = () => invoke<CliHistoryEntry[]>('cli_history_list')
export const cliHistoryGet = (id: string) => invoke<CliHistoryDetail>('cli_history_get', { id })

// Checkpoints (file history)
export const fileHistoryCheckpoints = (projectName: string, sessionId: string) =>
  invoke<Checkpoint[]>('file_history_checkpoints', { projectName, sessionId })
export const fileHistoryDiff = (sessionId: string, left: DiffSide, right: DiffSide) =>
  invoke<DiffResult>('file_history_diff', { sessionId, left, right })
export const fileHistoryBlob = (sessionId: string, backupFileName: string) =>
  invoke<string>('file_history_blob', { sessionId, backupFileName })
export const fileHistoryRestore = (
  projectName: string,
  sessionId: string,
  backupFileName: string,
  dest: string,
) => invoke<void>('file_history_restore', { projectName, sessionId, backupFileName, dest })

// Usage
export const usageRefresh = () => invoke<IndexStats>('usage_refresh')
export const usageRollup = (query: UsageQuery) => invoke<UsageReport>('usage_rollup', { query })
export const usageActivity = (days?: number) => invoke<ActivityDay[]>('usage_activity', { days })

// Jobs
export const jobsList = () => invoke<Job[]>('jobs_list')
export const jobsGet = (jobId: string) => invoke<JobDetail>('jobs_get', { jobId })

// Teams
export const teamsList = () => invoke<Team[]>('teams_list')
export const teamsGet = (id: string) => invoke<Team>('teams_get', { id })

// Hooks
export const hooksList = (workingDir?: string) =>
  invoke<HookGroup[]>('hooks_list', { workingDir })
export const hooksGet = (id: string, workingDir?: string) =>
  invoke<HookGroup>('hooks_get', { id, workingDir })
export const hooksCreate = (input: HookInput) => invoke<HookGroup>('hooks_create', { input })
export const hooksUpdate = (id: string, input: HookInput) =>
  invoke<HookGroup>('hooks_update', { id, input })
export const hooksDelete = (id: string, workingDir?: string) =>
  invoke<void>('hooks_delete', { id, workingDir })
export const hooksRawGet = (scope: SettingsScope, workingDir?: string) =>
  invoke<string>('hooks_raw_get', { scope, workingDir })
export const hooksRawPut = (
  scope: SettingsScope,
  content: string,
  workingDir?: string,
  expectedMtimeMs?: number,
) => invoke<number>('hooks_raw_put', { scope, workingDir, content, expectedMtimeMs })

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
export const projectsClaudeMdGet = (name: string) =>
  invoke<string>('projects_claude_md_get', { name })
export const projectsClaudeMdPut = (name: string, content: string) =>
  invoke<void>('projects_claude_md_put', { name, content })

// Sessions
export const sessionsThreads = (projectName: string, sessionId: string) =>
  invoke<Thread[]>('sessions_threads', { projectName, sessionId })
export const sessionsThreadMessages = (
  projectName: string,
  sessionId: string,
  threadId: string,
  afterIndex?: number,
  limit?: number,
) =>
  invoke<Page<Message>>('sessions_thread_messages', {
    projectName,
    sessionId,
    threadId,
    afterIndex,
    limit,
  })
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
export const settingsScopes = (workingDir?: string) =>
  invoke<ScopeInfo[]>('settings_scopes', { workingDir })
export const settingsRawGet = (scope: SettingsScope, workingDir?: string) =>
  invoke<RawDoc>('settings_raw_get', { scope, workingDir })
export const settingsRawPut = (
  scope: SettingsScope,
  content: string,
  workingDir?: string,
  expectedMtimeMs?: number,
) => invoke<number>('settings_raw_put', { scope, workingDir, content, expectedMtimeMs })
/** Send only the keys you own: unknown keys in the file are never touched. */
export const settingsPatch = (
  scope: SettingsScope,
  patch: Record<string, unknown>,
  workingDir?: string,
  expectedMtimeMs?: number,
) => invoke<number>('settings_patch', { scope, workingDir, patch, expectedMtimeMs })
export const settingsEffective = (workingDir?: string) =>
  invoke<EffectiveEntry[]>('settings_effective', { workingDir })

// Permissions
export const permissionsGet = (scope: SettingsScope, workingDir?: string) =>
  invoke<Permissions>('permissions_get', { scope, workingDir })
export const permissionsPut = (
  scope: SettingsScope,
  permissions: Permissions,
  workingDir?: string,
  expectedMtimeMs?: number,
) => invoke<number>('permissions_put', { scope, workingDir, permissions, expectedMtimeMs })
export const permissionsEffective = (workingDir?: string) =>
  invoke<EffectivePermissions>('permissions_effective', { workingDir })
export const permissionsValidate = (permissions: Permissions) =>
  invoke<RuleIssue[]>('permissions_validate', { permissions })

// Memory (CLAUDE.md and rules)
export const memoryList = (workingDir?: string) =>
  invoke<MemoryFile[]>('memory_list', { workingDir })
export const memoryAgentList = (workingDir?: string) =>
  invoke<MemoryFile[]>('memory_agent_list', { workingDir })
export const memoryGet = (id: string, workingDir?: string) =>
  invoke<MemoryDoc>('memory_get', { id, workingDir })
export const memoryPut = (
  id: string,
  content: string,
  workingDir?: string,
  expectedMtimeMs?: number,
) => invoke<MemoryDoc>('memory_put', { id, workingDir, content, expectedMtimeMs })
export const memoryDelete = (id: string, workingDir?: string) =>
  invoke<void>('memory_delete', { id, workingDir })
export const memoryPreview = (id: string, workingDir?: string) =>
  invoke<MemoryPreview>('memory_preview', { id, workingDir })

// Status line
export const statuslineGet = (scope: SettingsScope, workingDir?: string) =>
  invoke<StatusLine>('statusline_get', { scope, workingDir })
export const statuslinePut = (
  scope: SettingsScope,
  statusLine: StatusLine,
  workingDir?: string,
  expectedMtimeMs?: number,
) => invoke<number>('statusline_put', { scope, workingDir, statusLine, expectedMtimeMs })
export const statuslineDelete = (scope: SettingsScope, workingDir?: string) =>
  invoke<number>('statusline_delete', { scope, workingDir })
export const statuslinePreview = (statusLine: StatusLine, workingDir?: string) =>
  invoke<StatusLinePreview>('statusline_preview', { statusLine, workingDir })

// Keybindings
export const keybindingsGet = () => invoke<KeybindingsDoc>('keybindings_get')
export const keybindingsPut = (bindings: Keybinding[], expectedMtimeMs?: number) =>
  invoke<KeybindingsDoc>('keybindings_put', { bindings, expectedMtimeMs })
export const keybindingsCreate = () => invoke<KeybindingsDoc>('keybindings_create')
export const keybindingsRawGet = () => invoke<string>('keybindings_raw_get')
export const keybindingsRawPut = (content: string, expectedMtimeMs?: number) =>
  invoke<number>('keybindings_raw_put', { content, expectedMtimeMs })
export const keybindingsValidate = (chord: string) =>
  invoke<ChordValidation>('keybindings_validate', { chord })
export const configGet = () => invoke<AppConfig>('config_get')
export const configSet = (config: AppConfig) => invoke<void>('config_set', { config })
export const setupFinalize = (payload: SetupPayload) =>
  invoke<void>('setup_finalize', { payload })

// Claude directory explorer
export const claudeDirectoryTree = (projectPath?: string) =>
  invoke<ClaudeDirTree[]>('claude_directory_tree', { projectPath })

// Filesystem utilities
export const directoriesList = (parent: string) =>
  invoke<DirEntry[]>('directories_list', { parent })
export const filesRead = (path: string) => invoke<string>('files_read', { path })
export const fsHomeDir = () => invoke<string>('fs_home_dir')

// Debug
export const debugClaudeCli = () => invoke<ClaudeCliInfo | null>('debug_claude_cli')
export const appVersion = () => invoke<string>('app_version')
