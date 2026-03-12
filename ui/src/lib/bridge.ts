import type { FlowGraph, EvaluationTrace } from './types';

/** Whether we're running inside a Tauri desktop app. */
export const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

/** Load a DMN model from a file path via Tauri command. */
export async function loadDmnModel(path: string): Promise<FlowGraph> {
  if (!isTauri) {
    throw new Error('loadDmnModel requires the Tauri runtime');
  }
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<FlowGraph>('load_dmn_model', { path });
}

/** Evaluate a DMN model with input data via Tauri command. */
export async function evaluateWithTrace(
  modelPath: string,
  inputJson: string
): Promise<EvaluationTrace> {
  if (!isTauri) {
    throw new Error('evaluateWithTrace requires the Tauri runtime');
  }
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<EvaluationTrace>('evaluate_with_trace', { modelPath, inputJson });
}

/** Open a file dialog for selecting DMN files. */
export async function openFileDialog(): Promise<string | null> {
  if (!isTauri) return null;
  const { open } = await import('@tauri-apps/plugin-dialog');
  const result = await open({
    multiple: false,
    filters: [
      { name: 'DMN Files', extensions: ['dmn', 'xml'] },
      { name: 'All Files', extensions: ['*'] }
    ]
  });
  if (typeof result === 'string') return result;
  if (result && 'path' in result) return (result as { path: string }).path;
  return null;
}
