<script lang="ts">
  import type { RecentProject } from '$lib/types';

  let { projects, onSelectProject, onRemoveProject, onOpenProject, onNewProject } = $props<{
    projects: RecentProject[];
    onSelectProject: (project: RecentProject) => void;
    onRemoveProject: (path: string) => void;
    onOpenProject: () => void;
    onNewProject: () => void;
  }>();

  let selectedIndex = $state(0);

  function handleKeydown(e: KeyboardEvent) {
    if (projects.length === 0) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, projects.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (e.key === 'Enter') {
      e.preventDefault();
      onSelectProject(projects[selectedIndex]);
    }
  }

  function formatDate(isoStr: string): string {
    try {
      const d = new Date(isoStr);
      const now = new Date();
      const diffMs = now.getTime() - d.getTime();
      const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
      if (diffDays === 0) return 'Today';
      if (diffDays === 1) return 'Yesterday';
      if (diffDays < 7) return `${diffDays} days ago`;
      return d.toLocaleDateString();
    } catch {
      return isoStr;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="picker">
  <div class="picker-content">
    <div class="picker-header">
      <div class="brand-row">
        <span class="logo-text">DSNTK</span>
        <span class="logo-sub">Visual DMN Explorer</span>
      </div>
      <p class="subtitle">Pick up where you left off, or start something new.</p>
    </div>

    <div class="picker-actions">
      <button class="btn-action" onclick={onNewProject}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        New from template
      </button>
      <button class="btn-action" onclick={onOpenProject}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        Open a project folder
      </button>
    </div>

    {#if projects.length > 0}
      <div class="recent-section">
        <h3>Recent Projects</h3>
        <div class="project-list" role="listbox">
          {#each projects as project, i}
            <button
              class="project-row"
              class:selected={i === selectedIndex}
              role="option"
              aria-selected={i === selectedIndex}
              onclick={() => onSelectProject(project)}
              onmouseenter={() => selectedIndex = i}
            >
              <div class="project-icon">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                </svg>
              </div>
              <div class="project-info">
                <span class="project-name">{project.name}</span>
                <span class="project-path">{project.path}</span>
              </div>
              <span class="project-date">{formatDate(project.lastOpened)}</span>
              <span
                class="btn-remove"
                role="button"
                tabindex="-1"
                title="Remove from recent"
                onclick={(e) => { e.stopPropagation(); onRemoveProject(project.path); }}
                onkeydown={(e) => { if (e.key === 'Enter') { e.stopPropagation(); onRemoveProject(project.path); } }}
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </span>
            </button>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .picker {
    display: flex;
    justify-content: center;
    align-items: flex-start;
    height: 100%;
    overflow-y: auto;
    background: #0f0f23;
    padding: 48px 24px;
  }

  .picker-content {
    max-width: 600px;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }

  .picker-header {
    text-align: center;
  }

  .brand-row {
    display: flex;
    align-items: baseline;
    justify-content: center;
    gap: 10px;
    margin-bottom: 8px;
  }

  .logo-text {
    font-size: 28px;
    font-weight: 700;
    color: #8b5cf6;
    letter-spacing: 2px;
  }

  .logo-sub {
    font-size: 13px;
    color: #666;
  }

  .subtitle {
    font-size: 14px;
    color: #888;
  }

  .picker-actions {
    display: flex;
    justify-content: center;
    gap: 12px;
  }

  .btn-action {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 18px;
    background: #16213e;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #ccc;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-action:hover {
    background: #1a2a4e;
    border-color: rgba(139, 92, 246, 0.3);
    color: #eee;
  }

  .recent-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .recent-section h3 {
    font-size: 13px;
    font-weight: 600;
    color: #888;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .project-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    background: #16213e;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.06);
    overflow: hidden;
  }

  .project-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s ease;
    color: inherit;
    width: 100%;
  }

  .project-row:hover,
  .project-row.selected {
    background: rgba(139, 92, 246, 0.08);
  }

  .project-row.selected {
    outline: none;
  }

  .project-icon {
    color: #666;
    flex-shrink: 0;
  }

  .project-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .project-name {
    font-size: 13px;
    font-weight: 500;
    color: #eee;
  }

  .project-path {
    font-size: 11px;
    color: #555;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .project-date {
    font-size: 11px;
    color: #555;
    flex-shrink: 0;
  }

  .btn-remove {
    background: none;
    border: none;
    color: #555;
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    opacity: 0;
    transition: all 0.1s ease;
    flex-shrink: 0;
  }

  .project-row:hover .btn-remove {
    opacity: 1;
  }

  .btn-remove:hover {
    color: #ef4444;
    background: rgba(239, 68, 68, 0.1);
  }
</style>
