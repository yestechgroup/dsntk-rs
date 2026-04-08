import type { ModelsResponse, TraceResponse } from './types';

const BASE_URL = '/api/v1';

export async function fetchModels(): Promise<ModelsResponse> {
  const res = await fetch(`${BASE_URL}/models`);
  if (!res.ok) throw new Error(`Failed to fetch models: ${res.statusText}`);
  return res.json();
}

export async function evaluateTrace(path: string, inputs: Record<string, unknown>): Promise<TraceResponse> {
  const res = await fetch(`${BASE_URL}/evaluate-trace`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ path, inputs }),
  });
  if (!res.ok) throw new Error(`Evaluation failed: ${res.statusText}`);
  return res.json();
}
