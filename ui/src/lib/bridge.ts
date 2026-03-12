import type { FlowGraph, DecisionTableInfo } from './types';

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
