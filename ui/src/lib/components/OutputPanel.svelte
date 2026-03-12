<script lang="ts">
  import type { EvaluationTrace } from '$lib/types';
  import { STATUS_COLORS } from '$lib/types';

  let { trace } = $props<{ trace: EvaluationTrace | null }>();
</script>

<div class="output-panel">
  <h3>Evaluation Output</h3>
  {#if trace}
    <div class="output-value">
      <span class="label">Result:</span>
      <code>{trace.outputValue}</code>
    </div>
    <div class="node-traces">
      <h4>Node Traces</h4>
      {#each Object.entries(trace.nodeResults) as [nodeId, nodeTrace]}
        <div class="trace-item">
          <span
            class="status-dot"
            style="background: {STATUS_COLORS[nodeTrace.status] ?? '#d1d5db'};"
          ></span>
          <span class="trace-id" title={nodeId}>
            {nodeId.length > 20 ? nodeId.slice(0, 20) + '...' : nodeId}
          </span>
          <span class="trace-status">{nodeTrace.status}</span>
          <code class="trace-value">{nodeTrace.value}</code>
        </div>
      {/each}
    </div>
  {:else}
    <div class="placeholder">Run an evaluation to see results here.</div>
  {/if}
</div>

<style>
  .output-panel {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    overflow-y: auto;
  }

  h3 {
    font-size: 14px;
    font-weight: 600;
    color: #eee;
  }

  h4 {
    font-size: 12px;
    font-weight: 600;
    color: #aaa;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .output-value {
    background: rgba(255, 255, 255, 0.05);
    padding: 10px;
    border-radius: 6px;
  }

  .output-value .label {
    font-size: 12px;
    color: #aaa;
    margin-right: 8px;
  }

  .output-value code {
    font-family: 'Fira Code', monospace;
    color: #22c55e;
  }

  .node-traces {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .trace-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    padding: 4px 0;
  }

  .status-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .trace-id {
    color: #aaa;
    flex-shrink: 0;
  }

  .trace-status {
    font-weight: 600;
    text-transform: uppercase;
    font-size: 10px;
    flex-shrink: 0;
  }

  .trace-value {
    font-family: 'Fira Code', monospace;
    font-size: 11px;
    color: #ccc;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .placeholder {
    color: #666;
    font-style: italic;
    font-size: 13px;
  }
</style>
