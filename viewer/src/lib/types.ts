export type TraceNode =
  | { type: 'input_data'; id: string; name: string; value?: unknown }
  | { type: 'decision_table'; id: string; name: string; hit_policy: string; input_columns: string[]; output_columns: string[]; rules: TraceRule[] };

export interface TraceRule {
  index: number;
  input_entries: string[];
  output_entries: string[];
}

export interface TraceEdge {
  source: string;
  target: string;
  label: string;
}

export interface TraceGraph {
  nodes: TraceNode[];
  edges: TraceEdge[];
}

export interface CellEvaluation {
  rule_index: number;
  column_index: number;
  expression: string;
  input_value: unknown;
  result: boolean;
}

export interface TraceStep {
  node_id: string;
  input_values: Record<string, unknown>;
  matched_rules: number[];
  output_value: unknown;
  cell_evaluations: CellEvaluation[];
}

export interface EvaluationTrace {
  graph: TraceGraph;
  evaluation_order: string[];
  steps: TraceStep[];
}

export interface TraceResponse {
  data: unknown;
  trace?: EvaluationTrace;
}

export interface InputInfo {
  name: string;
  feel_type: string;
  allowed_values?: string[];
  optional: boolean;
}

export interface ModelInfo {
  namespace: string;
  name: string;
  invocables: string[];
  inputs: InputInfo[];
}

export interface ModelsResponse {
  models: ModelInfo[];
}

export type NodeEvalState = 'unevaluated' | 'evaluating' | 'evaluated';
export type EdgeEvalState = 'inactive' | 'animating' | 'completed';
