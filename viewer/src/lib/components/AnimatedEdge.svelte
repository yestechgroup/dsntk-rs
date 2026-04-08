<script lang="ts">
  import { BaseEdge, getBezierPath, type EdgeProps } from '@xyflow/svelte';
  import { evaluatedNodeIds, evaluatingNodeId } from '$lib/stores';

  type $$Props = EdgeProps & {
    data?: { label?: string } | undefined;
  };

  export let id: string;
  export let sourceX: number;
  export let sourceY: number;
  export let targetX: number;
  export let targetY: number;
  export let sourcePosition: any;
  export let targetPosition: any;
  export let data: { label?: string } | undefined = undefined;
  export let target: string = '';

  $: [path, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
  });

  $: isCompleted = $evaluatedNodeIds.has(target);
  $: isAnimating = $evaluatingNodeId === target;

  $: style = isCompleted
    ? 'stroke: #3fb950; stroke-width: 2;'
    : isAnimating
      ? 'stroke: #d29922; stroke-width: 2;'
      : 'stroke: #30363d; stroke-width: 1.5; stroke-dasharray: 5 5;';
</script>

<BaseEdge {id} {path} {style} />

{#if isAnimating}
  <circle r="4" fill="#d29922">
    <animateMotion dur="0.8s" repeatCount="indefinite" {path} />
  </circle>
{/if}

{#if isCompleted && data?.label}
  <foreignObject
    x={labelX - 40}
    y={labelY - 10}
    width="80"
    height="20"
  >
    <div class="edge-label">→ {data.label}</div>
  </foreignObject>
{/if}

<style>
  .edge-label {
    background: #1a3a1a;
    color: #3fb950;
    font-size: 9px;
    font-family: monospace;
    padding: 2px 6px;
    border-radius: 4px;
    text-align: center;
    white-space: nowrap;
  }
</style>
