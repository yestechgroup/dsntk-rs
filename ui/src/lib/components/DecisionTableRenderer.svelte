<script lang="ts">
  import type { DecisionTableInfo } from '$lib/types';

  let { table } = $props<{ table: DecisionTableInfo | null }>();
</script>

<div class="dt-panel">
  {#if table}
    <h3>Decision Table — {table.nodeId}</h3>
    <div class="dt-meta">
      <span class="hit-policy">Hit Policy: <strong>{table.hitPolicy}</strong></span>
    </div>
    <div class="dt-scroll">
      <table>
        <thead>
          <tr>
            <th class="rule-col">#</th>
            {#each table.inputColumns as col}
              <th class="input-col">{col || '(input)'}</th>
            {/each}
            {#each table.outputColumns as col}
              <th class="output-col">{col || '(output)'}</th>
            {/each}
          </tr>
        </thead>
        <tbody>
          {#each table.rules as rule}
            <tr>
              <td class="rule-col">{rule.index + 1}</td>
              {#each rule.inputEntries as entry}
                <td class="input-col">{entry || '-'}</td>
              {/each}
              {#each rule.outputEntries as entry}
                <td class="output-col">{entry}</td>
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else}
    <div class="placeholder">Select a decision node to view its decision table.</div>
  {/if}
</div>

<style>
  .dt-panel {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    overflow: hidden;
  }

  h3 {
    font-size: 14px;
    font-weight: 600;
    color: #eee;
  }

  .dt-meta {
    font-size: 12px;
    color: #aaa;
  }

  .hit-policy strong {
    color: #8b5cf6;
  }

  .dt-scroll {
    overflow: auto;
    max-height: 400px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
    font-family: 'Fira Code', monospace;
  }

  th, td {
    padding: 6px 10px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    white-space: nowrap;
  }

  th {
    background: rgba(255, 255, 255, 0.05);
    font-weight: 600;
    text-align: left;
  }

  .rule-col {
    color: #666;
    width: 30px;
    text-align: center;
  }

  .input-col {
    color: #3b82f6;
  }

  th.input-col {
    background: rgba(59, 130, 246, 0.1);
  }

  .output-col {
    color: #22c55e;
  }

  th.output-col {
    background: rgba(34, 197, 94, 0.1);
  }

  .placeholder {
    color: #666;
    font-style: italic;
    font-size: 13px;
  }
</style>
