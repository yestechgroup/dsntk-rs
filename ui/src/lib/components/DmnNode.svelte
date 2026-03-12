<script lang="ts">
  import { Handle, Position } from '@xyflow/svelte';
  import { NODE_TYPE_COLORS, STATUS_COLORS } from '$lib/types';

  let { data } = $props<{ data: {
    label: string;
    nodeType: string;
    dataTypeRef?: string | null;
    status?: string;
    value?: string;
  } }>();

  let borderColor = $derived(
    data.status && data.status !== 'pending'
      ? STATUS_COLORS[data.status] ?? '#d1d5db'
      : NODE_TYPE_COLORS[data.nodeType] ?? '#6b7280'
  );

  let bgColor = $derived(
    data.status === 'hit' ? 'rgba(34, 197, 94, 0.15)' :
    data.status === 'miss' ? 'rgba(239, 68, 68, 0.15)' :
    data.status === 'ignored' ? 'rgba(156, 163, 175, 0.1)' :
    'rgba(255, 255, 255, 0.05)'
  );

  let typeLabel = $derived(
    data.nodeType === 'inputData' ? 'Input Data' :
    data.nodeType === 'decision' ? 'Decision' :
    data.nodeType === 'bkm' ? 'BKM' :
    data.nodeType === 'knowledgeSource' ? 'Knowledge Source' :
    data.nodeType
  );

  let shapeClass = $derived(
    data.nodeType === 'inputData' ? 'shape-rounded' :
    data.nodeType === 'bkm' ? 'shape-clipped' :
    data.nodeType === 'knowledgeSource' ? 'shape-wavy' :
    'shape-rect'
  );
</script>

<div class="dmn-node {shapeClass}" style="border-color: {borderColor}; background: {bgColor};">
  <Handle type="target" position={Position.Top} />
  <div class="node-type" style="color: {borderColor};">{typeLabel}</div>
  <div class="node-label">{data.label}</div>
  {#if data.dataTypeRef}
    <div class="node-type-ref">{data.dataTypeRef}</div>
  {/if}
  {#if data.value && data.status !== 'pending'}
    <div class="node-value" title={data.value}>
      {data.value.length > 30 ? data.value.slice(0, 30) + '...' : data.value}
    </div>
  {/if}
  <Handle type="source" position={Position.Bottom} />
</div>

<style>
  .dmn-node {
    padding: 12px 16px;
    min-width: 150px;
    max-width: 250px;
    border: 2px solid;
    font-family: 'Segoe UI', system-ui, sans-serif;
    color: #eee;
    text-align: center;
  }

  .shape-rect {
    border-radius: 4px;
  }

  .shape-rounded {
    border-radius: 20px;
  }

  .shape-clipped {
    border-radius: 4px;
    clip-path: polygon(10% 0%, 90% 0%, 100% 50%, 90% 100%, 10% 100%, 0% 50%);
    padding: 12px 24px;
  }

  .shape-wavy {
    border-radius: 2px;
    border-style: dashed;
  }

  .node-type {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }

  .node-label {
    font-size: 13px;
    font-weight: 500;
  }

  .node-type-ref {
    font-size: 10px;
    color: #8b5cf6;
    font-family: 'Fira Code', monospace;
    margin-top: 2px;
  }

  .node-value {
    font-size: 11px;
    margin-top: 6px;
    padding-top: 6px;
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    color: #aaa;
    font-family: monospace;
  }
</style>
