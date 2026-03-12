import type { FlowGraph, DecisionTableInfo, TemplateInfo, RecentProject } from './types';

/** Whether we're running inside a Tauri desktop app. */
export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

/** Load a markdown DMN project directory via Tauri command. */
export async function loadDmnProject(projectDir: string): Promise<FlowGraph> {
  if (!isTauri) {
    throw new Error('loadDmnProject requires the Tauri runtime');
  }
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<FlowGraph>('load_dmn_project', { projectDir });
}

/** Parse a decision table from a markdown file via Tauri command. */
export async function parseDecisionTable(filePath: string): Promise<DecisionTableInfo> {
  if (!isTauri) {
    throw new Error('parseDecisionTable requires the Tauri runtime');
  }
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<DecisionTableInfo>('parse_decision_table', { filePath });
}

/** Evaluate a FEEL expression via Tauri command. */
export async function evaluateFeelExpression(
  expression: string,
  contextJson: string
): Promise<string> {
  if (!isTauri) {
    throw new Error('evaluateFeelExpression requires the Tauri runtime');
  }
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<string>('evaluate_feel_expression', { expression, contextJson });
}

/** Open a directory dialog for selecting a DMN project folder. */
export async function openProjectDialog(): Promise<string | null> {
  if (!isTauri) return null;
  const { open } = await import('@tauri-apps/plugin-dialog');
  const result = await open({
    directory: true,
    multiple: false,
    title: 'Open Markdown DMN Project'
  });
  if (typeof result === 'string') return result;
  return null;
}

/** List all available project templates. */
export async function listTemplates(): Promise<TemplateInfo[]> {
  if (!isTauri) return [];
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<TemplateInfo[]>('list_templates');
}

/** Create a new project from a template. */
export async function createProjectFromTemplate(
  templateName: string,
  destDir: string
): Promise<string> {
  if (!isTauri) {
    throw new Error('createProjectFromTemplate requires the Tauri runtime');
  }
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<string>('create_project_from_template', { templateName, destDir });
}

/** Get recently opened projects. */
export async function getRecentProjects(): Promise<RecentProject[]> {
  if (!isTauri) return [];
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<RecentProject[]>('get_recent_projects');
}

/** Add or update a project in the recent projects list. */
export async function addRecentProject(path: string, name: string): Promise<void> {
  if (!isTauri) return;
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<void>('add_recent_project', { path, name });
}

/** Remove a project from recent projects. */
export async function removeRecentProject(path: string): Promise<void> {
  if (!isTauri) return;
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<void>('remove_recent_project', { path });
}

/** Open a save dialog for choosing where to create a new project. */
export async function openSaveDialog(defaultName: string): Promise<string | null> {
  if (!isTauri) return null;
  const { open } = await import('@tauri-apps/plugin-dialog');
  const result = await open({
    directory: true,
    multiple: false,
    title: `Choose location for "${defaultName}"`
  });
  if (typeof result === 'string') return result;
  return null;
}
