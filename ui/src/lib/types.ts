/** A node in the DMN flow graph, parsed from markdown front matter. */
export interface FlowNode {
  id: string;
  label: string;
  nodeType: string;
  /** TypeScript type reference (e.g. "ApplicantData"). */
  dataTypeRef: string | null;
  /** Path to schema file (e.g. "../types/loan.ts"). */
  schemaPath: string | null;
  /** Markdown body content (documentation + decision table). */
  body: string;
  /** Source file path relative to project root. */
  sourceFile: string;
}

/** An edge in the DMN flow graph. */
export interface FlowEdge {
  id: string;
  source: string;
  target: string;
  /** "requires" or "governed-by". */
  edgeType: string;
}

/** A TypeScript type file from the project. */
export interface TypeFile {
  path: string;
  content: string;
}

/** The complete flow graph for SvelteFlow rendering. */
export interface FlowGraph {
  nodes: FlowNode[];
  edges: FlowEdge[];
  projectName: string;
  typeFiles: TypeFile[];
}

/** Parsed decision table info. */
export interface DecisionTableInfo {
  nodeId: string;
  hitPolicy: string;
  inputColumns: string[];
  outputColumns: string[];
  rules: DecisionRuleInfo[];
}

/** A single rule row from a decision table. */
export interface DecisionRuleInfo {
  index: number;
  inputEntries: string[];
  outputEntries: string[];
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
  bkm: '#f59e0b',
  knowledgeSource: '#ec4899'
};
