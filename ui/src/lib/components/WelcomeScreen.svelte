<script lang="ts">
  import type { TemplateInfo } from '$lib/types';

  let { templates, onSelectTemplate, onOpenProject } = $props<{
    templates: TemplateInfo[];
    onSelectTemplate: (template: TemplateInfo) => void;
    onOpenProject: () => void;
  }>();

  const TEMPLATE_ICONS: Record<string, string> = {
    'loan-eligibility': 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z',
    'insurance-pricing': 'M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4z',
    'tax-calculator': 'M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-5 14H7v-2h7v2zm3-4H7v-2h10v2zm0-4H7V7h10v2z',
    'order-routing': 'M20 8h-3V4H3c-1.1 0-2 .9-2 2v11h2c0 1.66 1.34 3 3 3s3-1.34 3-3h6c0 1.66 1.34 3 3 3s3-1.34 3-3h2v-5l-3-4z',
    'compliance-checker': 'M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41z',
    'scorecard': 'M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z',
  };
</script>

<div class="welcome">
  <div class="welcome-content">
    <div class="hero">
      <div class="brand">
        <span class="logo-text">DSNTK</span>
        <span class="logo-sub">Visual DMN Explorer</span>
      </div>
      <p class="tagline">Create and explore decision models using Markdown-native DMN projects.</p>
    </div>

    <div class="actions-row">
      <button class="btn-open-existing" onclick={onOpenProject}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        Open existing project
      </button>
    </div>

    <div class="templates-section">
      <h2>Start from a template</h2>
      <p class="templates-hint">Choose a template to scaffold a new Markdown DMN project.</p>
      <div class="template-gallery">
        {#each templates as tmpl}
          <button class="template-card" onclick={() => onSelectTemplate(tmpl)}>
            <div class="card-icon">
              <svg width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
                <path d={TEMPLATE_ICONS[tmpl.name] ?? 'M12 2l-5.5 9h11z'} />
              </svg>
            </div>
            <div class="card-body">
              <h3>{tmpl.name}</h3>
              <p class="card-desc">{tmpl.description}</p>
              <div class="card-meta">
                <span class="node-count">{tmpl.nodeCount} nodes</span>
                <div class="features">
                  {#each tmpl.features as feat}
                    <span class="feature-tag">{feat}</span>
                  {/each}
                </div>
              </div>
            </div>
          </button>
        {/each}
      </div>
    </div>
  </div>
</div>

<style>
  .welcome {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    height: 100%;
    overflow-y: auto;
    background: #0f0f23;
    padding: 48px 24px;
  }

  .welcome-content {
    max-width: 860px;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .hero {
    text-align: center;
  }

  .brand {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    margin-bottom: 12px;
  }

  .logo-text {
    font-size: 32px;
    font-weight: 700;
    color: #8b5cf6;
    letter-spacing: 2px;
  }

  .logo-sub {
    font-size: 14px;
    color: #666;
    font-weight: 400;
    letter-spacing: 0.5px;
  }

  .tagline {
    font-size: 15px;
    color: #888;
    line-height: 1.5;
  }

  .actions-row {
    display: flex;
    justify-content: center;
    gap: 12px;
  }

  .btn-open-existing {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 20px;
    background: #16213e;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #ccc;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-open-existing:hover {
    background: #1a2a4e;
    border-color: rgba(139, 92, 246, 0.3);
    color: #eee;
  }

  .templates-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .templates-section h2 {
    font-size: 16px;
    font-weight: 600;
    color: #eee;
    text-align: center;
  }

  .templates-hint {
    font-size: 13px;
    color: #666;
    text-align: center;
    margin-bottom: 4px;
  }

  .template-gallery {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 12px;
  }

  .template-card {
    display: flex;
    gap: 12px;
    padding: 16px;
    background: #16213e;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 8px;
    cursor: pointer;
    text-align: left;
    transition: all 0.2s ease;
    color: inherit;
  }

  .template-card:hover {
    border-color: rgba(139, 92, 246, 0.4);
    background: #1a2a4e;
    transform: translateY(-2px);
  }

  .card-icon {
    flex-shrink: 0;
    width: 44px;
    height: 44px;
    border-radius: 8px;
    background: rgba(139, 92, 246, 0.15);
    color: #8b5cf6;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
  }

  .card-body h3 {
    font-size: 13px;
    font-weight: 600;
    color: #eee;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-desc {
    font-size: 11px;
    color: #888;
    line-height: 1.4;
  }

  .card-meta {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 4px;
  }

  .node-count {
    font-size: 10px;
    color: #666;
  }

  .features {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .feature-tag {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 3px;
    background: rgba(139, 92, 246, 0.1);
    color: #8b5cf6;
  }
</style>
