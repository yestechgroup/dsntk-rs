<script lang="ts">
  let { inputJson = '{}', onEvaluate, disabled = false } = $props<{
    inputJson: string;
    onEvaluate: (json: string) => void;
    disabled?: boolean;
  }>();

  let localJson = $state(inputJson);
  let parseError = $state('');

  function handleEvaluate() {
    try {
      JSON.parse(localJson);
      parseError = '';
      onEvaluate(localJson);
    } catch (e) {
      parseError = 'Invalid JSON';
    }
  }
</script>

<div class="input-panel">
  <h3>Input Data (JSON)</h3>
  <textarea
    bind:value={localJson}
    placeholder='{"key": "value"}'
    rows={8}
    spellcheck={false}
  ></textarea>
  {#if parseError}
    <div class="error">{parseError}</div>
  {/if}
  <button onclick={handleEvaluate} disabled={disabled}>
    Evaluate
  </button>
</div>

<style>
  .input-panel {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  h3 {
    font-size: 14px;
    font-weight: 600;
    color: #eee;
  }

  textarea {
    width: 100%;
    background: rgba(255, 255, 255, 0.05);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 6px;
    color: #eee;
    padding: 10px;
    font-family: 'Fira Code', 'Cascadia Code', monospace;
    font-size: 13px;
    resize: vertical;
  }

  textarea:focus {
    outline: none;
    border-color: #8b5cf6;
  }

  .error {
    color: #ef4444;
    font-size: 12px;
  }

  button {
    background: #8b5cf6;
    color: #fff;
    align-self: flex-start;
  }

  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
