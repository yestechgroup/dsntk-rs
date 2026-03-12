<script lang="ts">
  import { SvelteFlow, Background, Controls, MiniMap } from '@xyflow/svelte';
  import type { Node, Edge } from '@xyflow/svelte';
  import DmnNode from '$lib/components/DmnNode.svelte';
  import InputPanel from '$lib/components/InputPanel.svelte';
  import OutputPanel from '$lib/components/OutputPanel.svelte';
  import { loadDmnModel, evaluateWithTrace, openFileDialog, isTauri } from '$lib/bridge';
  import { NODE_TYPE_COLORS } from '$lib/types';
  import type { FlowGraph, EvaluationTrace } from '$lib/types';

  const nodeTypes = { dmn: DmnNode };

  let nodes = $state<Node[]>([]);
  let edges = $state<Edge[]>([]);
  let modelPath = $state<string | null>(null);
  let modelName = $state<string>('');
  let trace = $state<EvaluationTrace | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);

  /** Apply auto-layout using a simple hierarchical algorithm. */
  function autoLayout(graph: FlowGraph): { nodes: Node[]; edges: Edge[] } {
    // Build adjacency: for each node, determine its depth (longest path from any root)
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

    // BFS from roots (nodes with no incoming) to assign depths
    const depth = new Map<string, number>();
    const roots = graph.nodes.filter((n) => (incomingEdges.get(n.id)?.length ?? 0) === 0);

    // Initialize all nodes with depth 0
    for (const n of graph.nodes) depth.set(n.id, 0);

    // Assign max-depth via topological traversal
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

    // Group nodes by depth
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
      style: `stroke: ${e.edgeType === 'knowledge' ? '#f59e0b' : '#6b7280'}; stroke-width: 1.5;`
    }));

    return { nodes: layoutNodes, edges: layoutEdges };
  }

  /** Handle file open. */
  async function handleOpen() {
    error = null;
    if (!isTauri) {
      error = 'File open requires the Tauri desktop runtime.';
      return;
    }
    const path = await openFileDialog();
    if (!path) return;

    loading = true;
    try {
      const graph = await loadDmnModel(path);
      modelPath = path;
      modelName = graph.modelName;
      trace = null;
      const layout = autoLayout(graph);
      nodes = layout.nodes;
      edges = layout.edges;
    } catch (e: any) {
      error = e.message ?? String(e);
    } finally {
      loading = false;
    }
  }

  /** Handle evaluation. */
  async function handleEvaluate(inputJson: string) {
    if (!modelPath) return;
    error = null;
    loading = true;
    try {
      trace = await evaluateWithTrace(modelPath, inputJson);

      // Update node colors based on trace
      nodes = nodes.map((node) => {
        const nodeTrace = trace?.nodeResults[node.id];
        return {
          ...node,
          data: {
            ...node.data,
            status: nodeTrace?.status ?? 'pending',
            value: nodeTrace?.value ?? ''
          }
        };
      });

      // Animate edges connected to hit nodes
      edges = edges.map((edge) => {
        const sourceTrace = trace?.nodeResults[edge.source];
        const targetTrace = trace?.nodeResults[edge.target];
        const isActive = sourceTrace?.status === 'hit' && targetTrace?.status === 'hit';
        return {
          ...edge,
          animated: isActive
        };
      });
    } catch (e: any) {
      error = e.message ?? String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="app-layout">
  <!-- Toolbar -->
  <header class="toolbar">
    <div class="toolbar-left">
      <span class="logo">DSNTK</span>
      <span class="separator">|</span>
      <span class="title">Visual DMN Explorer</span>
      {#if modelName}
        <span class="separator">-</span>
        <span class="model-name">{modelName}</span>
      {/if}
    </div>
    <div class="toolbar-right">
      <button class="btn-open" onclick={handleOpen} disabled={loading}>
        Open DMN Model
      </button>
    </div>
  </header>

  {#if error}
    <div class="error-bar">{error}</div>
  {/if}

  <!-- Main content -->
  <div class="main-content">
    <!-- Canvas -->
    <div class="canvas-area">
      {#if nodes.length > 0}
        <SvelteFlow {nodes} {edges} {nodeTypes} fitView>
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
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <polyline points="14 2 14 8 20 8" />
              <line x1="16" y1="13" x2="8" y2="13" />
              <line x1="16" y1="17" x2="8" y2="17" />
              <polyline points="10 9 9 9 8 9" />
            </svg>
          </div>
          <h2>Open a DMN Model</h2>
          <p>Click "Open DMN Model" to load a .dmn or .xml file and visualize its decision graph.</p>
        </div>
      {/if}
    </div>

    <!-- Side panel -->
    <aside class="side-panel">
      <InputPanel inputJson="{{}}" onEvaluate={handleEvaluate} disabled={!modelPath || loading} />
      <div class="divider"></div>
      <OutputPanel {trace} />
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

  .model-name {
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
    max-width: 300px;
    text-align: center;
    line-height: 1.5;
  }

  .side-panel {
    width: 320px;
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
</style>
