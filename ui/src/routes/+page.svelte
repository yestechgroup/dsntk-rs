<script lang="ts">
  import { SvelteFlow, Background, Controls, MiniMap } from '@xyflow/svelte';
  import type { Node, Edge } from '@xyflow/svelte';
  import DmnNode from '$lib/components/DmnNode.svelte';
  import DecisionTableRenderer from '$lib/components/DecisionTableRenderer.svelte';
  import { loadDmnProject, parseDecisionTable, openProjectDialog, isTauri } from '$lib/bridge';
  import { NODE_TYPE_COLORS } from '$lib/types';
  import type { FlowGraph, FlowNode, DecisionTableInfo, TypeFile } from '$lib/types';

  const nodeTypes = { dmn: DmnNode };

  let nodes = $state<Node[]>([]);
  let edges = $state<Edge[]>([]);
  let projectDir = $state<string | null>(null);
  let projectName = $state<string>('');
  let typeFiles = $state<TypeFile[]>([]);
  let selectedTable = $state<DecisionTableInfo | null>(null);
  let selectedNodeBody = $state<string>('');
  let loading = $state(false);
  let error = $state<string | null>(null);

  /** Apply auto-layout using a simple hierarchical algorithm. */
  function autoLayout(graph: FlowGraph): { nodes: Node[]; edges: Edge[] } {
    const nodeMap = new Map(graph.nodes.map((n) => [n.id, n]));
    const incomingEdges = new Map<string, string[]>();
    const outgoingEdges = new Map<string, string[]>();

    for (const n of graph.nodes) {
      incomingEdges.set(n.id, []);
      outgoingEdges.set(n.id, []);
    }

    for (const e of graph.edges) {
      incomingEdges.get(e.target)?.push(e.source);
      outgoingEdges.get(e.source)?.push(e.target);
    }

    const depth = new Map<string, number>();
    const roots = graph.nodes.filter((n) => (incomingEdges.get(n.id)?.length ?? 0) === 0);
    for (const n of graph.nodes) depth.set(n.id, 0);

    const queue = [...roots.map((r) => r.id)];
    const visited = new Set<string>();
    while (queue.length > 0) {
      const current = queue.shift()!;
      if (visited.has(current)) continue;
      visited.add(current);
      const currentDepth = depth.get(current) ?? 0;
      for (const target of outgoingEdges.get(current) ?? []) {
        const newDepth = currentDepth + 1;
        if (newDepth > (depth.get(target) ?? 0)) {
          depth.set(target, newDepth);
        }
        queue.push(target);
      }
    }

    const layers = new Map<number, string[]>();
    for (const [id, d] of depth) {
      if (!layers.has(d)) layers.set(d, []);
      layers.get(d)!.push(id);
    }

    const LAYER_HEIGHT = 150;
    const NODE_WIDTH = 200;
    const NODE_GAP = 40;

    const layoutNodes: Node[] = [];
    for (const [layerDepth, nodeIds] of layers) {
      const totalWidth = nodeIds.length * NODE_WIDTH + (nodeIds.length - 1) * NODE_GAP;
      const startX = -totalWidth / 2;
      nodeIds.forEach((id, i) => {
        const flowNode = nodeMap.get(id);
        if (!flowNode) return;
        layoutNodes.push({
          id,
          type: 'dmn',
          position: {
            x: startX + i * (NODE_WIDTH + NODE_GAP),
            y: layerDepth * LAYER_HEIGHT
          },
          data: {
            label: flowNode.label,
            nodeType: flowNode.nodeType,
            dataTypeRef: flowNode.dataTypeRef,
            sourceFile: flowNode.sourceFile,
            body: flowNode.body,
            status: 'pending',
            value: ''
          }
        });
      });
    }

    const layoutEdges: Edge[] = graph.edges.map((e) => ({
      id: e.id,
      source: e.source,
      target: e.target,
      type: 'default',
      animated: false,
      style: `stroke: ${e.edgeType === 'governed-by' ? '#ec4899' : '#6b7280'}; stroke-width: 1.5;${e.edgeType === 'governed-by' ? ' stroke-dasharray: 5 3;' : ''}`
    }));

    return { nodes: layoutNodes, edges: layoutEdges };
  }

  /** Handle project open — opens a directory, not a file. */
  async function handleOpen() {
    error = null;
    if (!isTauri) {
      error = 'Opening a project requires the Tauri desktop runtime.';
      return;
    }
    const dir = await openProjectDialog();
    if (!dir) return;

    loading = true;
    try {
      const graph = await loadDmnProject(dir);
      projectDir = dir;
      projectName = graph.projectName;
      typeFiles = graph.typeFiles;
      selectedTable = null;
      selectedNodeBody = '';
      const layout = autoLayout(graph);
      nodes = layout.nodes;
      edges = layout.edges;
    } catch (e: any) {
      error = e.message ?? String(e);
    } finally {
      loading = false;
    }
  }

  /** Handle node click — show decision table if it's a decision node. */
  async function handleNodeClick(event: CustomEvent) {
    const node = event.detail.node;
    if (!node || !projectDir) return;

    selectedNodeBody = node.data?.body ?? '';

    // Only parse decision tables for decision/bkm nodes
    if (node.data?.nodeType === 'decision' || node.data?.nodeType === 'bkm') {
      const sourceFile = node.data?.sourceFile;
      if (sourceFile) {
        try {
          const filePath = `${projectDir}/${sourceFile}`;
          selectedTable = await parseDecisionTable(filePath);
        } catch {
          // Node may not have a decision table (e.g. FEEL expression)
          selectedTable = null;
        }
      }
    } else {
      selectedTable = null;
    }
  }
</script>

<div class="app-layout">
  <header class="toolbar">
    <div class="toolbar-left">
      <span class="logo">DSNTK</span>
      <span class="separator">|</span>
      <span class="title">Visual DMN Explorer</span>
      {#if projectName}
        <span class="separator">-</span>
        <span class="project-name">{projectName}</span>
      {/if}
    </div>
    <div class="toolbar-right">
      <button class="btn-open" onclick={handleOpen} disabled={loading}>
        Open Project
      </button>
    </div>
  </header>

  {#if error}
    <div class="error-bar">{error}</div>
  {/if}

  <div class="main-content">
    <div class="canvas-area">
      {#if nodes.length > 0}
        <SvelteFlow {nodes} {edges} {nodeTypes} fitView on:nodeclick={handleNodeClick}>
          <Background />
          <Controls />
          <MiniMap
            nodeColor={(node) => NODE_TYPE_COLORS[node.data?.nodeType] ?? '#6b7280'}
          />
        </SvelteFlow>
      {:else}
        <div class="empty-state">
          <div class="empty-icon">
            <svg width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
          </div>
          <h2>Open a Markdown DMN Project</h2>
          <p>Click "Open Project" to load a project directory created via <code>dsntk new</code>.</p>
          <p class="hint">Projects contain <code>decisions/*.md</code> files with YAML front matter and <code>types/*.ts</code> schema definitions.</p>
        </div>
      {/if}
    </div>

    <aside class="side-panel">
      {#if selectedNodeBody}
        <div class="body-section">
          <h3>Documentation</h3>
          <div class="body-content">{selectedNodeBody}</div>
        </div>
        <div class="divider"></div>
      {/if}

      <DecisionTableRenderer table={selectedTable} />

      {#if typeFiles.length > 0}
        <div class="divider"></div>
        <div class="types-section">
          <h3>Type Definitions</h3>
          {#each typeFiles as tf}
            <details>
              <summary>{tf.path}</summary>
              <pre><code>{tf.content}</code></pre>
            </details>
          {/each}
        </div>
      {/if}
    </aside>
  </div>
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: #16213e;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    flex-shrink: 0;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .logo {
    font-weight: 700;
    font-size: 16px;
    color: #8b5cf6;
    letter-spacing: 1px;
  }

  .separator {
    color: #444;
  }

  .title {
    font-size: 14px;
    color: #aaa;
  }

  .project-name {
    font-size: 14px;
    color: #eee;
    font-weight: 500;
  }

  .toolbar-right {
    display: flex;
    gap: 8px;
  }

  .btn-open {
    background: #0f3460;
    color: #eee;
  }

  .error-bar {
    background: rgba(239, 68, 68, 0.2);
    color: #ef4444;
    padding: 8px 16px;
    font-size: 13px;
    border-bottom: 1px solid rgba(239, 68, 68, 0.3);
    flex-shrink: 0;
  }

  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .canvas-area {
    flex: 1;
    position: relative;
    background: #0f0f23;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #666;
    gap: 12px;
  }

  .empty-icon {
    opacity: 0.3;
  }

  .empty-state h2 {
    font-size: 18px;
    font-weight: 500;
    color: #888;
  }

  .empty-state p {
    font-size: 13px;
    max-width: 350px;
    text-align: center;
    line-height: 1.5;
  }

  .empty-state code {
    background: rgba(139, 92, 246, 0.15);
    padding: 2px 5px;
    border-radius: 3px;
    color: #8b5cf6;
    font-size: 12px;
  }

  .hint {
    color: #555;
    font-size: 12px !important;
  }

  .side-panel {
    width: 360px;
    background: #16213e;
    border-left: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    overflow-y: auto;
    flex-shrink: 0;
  }

  .divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.1);
    margin: 0 16px;
  }

  .body-section {
    padding: 16px;
  }

  .body-section h3 {
    font-size: 14px;
    font-weight: 600;
    color: #eee;
    margin-bottom: 8px;
  }

  .body-content {
    font-size: 13px;
    color: #aaa;
    line-height: 1.5;
    white-space: pre-wrap;
    max-height: 200px;
    overflow-y: auto;
  }

  .types-section {
    padding: 16px;
  }

  .types-section h3 {
    font-size: 14px;
    font-weight: 600;
    color: #eee;
    margin-bottom: 8px;
  }

  details {
    margin-bottom: 8px;
  }

  summary {
    font-size: 12px;
    color: #8b5cf6;
    cursor: pointer;
    padding: 4px 0;
  }

  pre {
    background: rgba(0, 0, 0, 0.3);
    border-radius: 4px;
    padding: 10px;
    overflow-x: auto;
    margin-top: 4px;
  }

  pre code {
    font-size: 11px;
    font-family: 'Fira Code', monospace;
    color: #ccc;
  }
</style>
