/** A node in the DMN flow graph. */
export interface FlowNode {
  id: string;
  label: string;
  nodeType: string;
}

/** An edge in the DMN flow graph. */
export interface FlowEdge {
  id: string;
  source: string;
  target: string;
  edgeType: string;
}

/** The complete flow graph for SvelteFlow rendering. */
export interface FlowGraph {
  nodes: FlowNode[];
  edges: FlowEdge[];
  modelName: string;
  modelNamespace: string;
}

/** Trace result for a single node. */
export interface NodeTrace {
  status: 'hit' | 'miss' | 'ignored' | 'pending';
  value: string;
}

/** Evaluation trace for the entire model. */
export interface EvaluationTrace {
  nodeResults: Record<string, NodeTrace>;
  outputValue: string;
}

/** Status color mapping. */
export const STATUS_COLORS: Record<string, string> = {
  hit: '#22c55e',
  miss: '#ef4444',
  ignored: '#9ca3af',
  pending: '#d1d5db'
};

/** Node type color mapping. */
export const NODE_TYPE_COLORS: Record<string, string> = {
  inputData: '#3b82f6',
  decision: '#8b5cf6',
  businessKnowledgeModel: '#f59e0b',
  decisionService: '#06b6d4',
  knowledgeSource: '#ec4899'
};
