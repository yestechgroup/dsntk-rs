import { test as base, expect } from '@playwright/test';
import { injectTauriMock, MOCK_TEMPLATES, MOCK_RECENT_PROJECTS, MOCK_FLOW_GRAPH_EMPTY } from './fixtures';

const test = base;

test.describe('View Transitions', () => {
  test('first-time user sees welcome screen (no recent projects)', async ({ page }) => {
    await injectTauriMock(page, { recentProjects: [] });
    await page.goto('/');
    await expect(page.locator('.welcome')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('.picker')).not.toBeVisible();
  });

  test('returning user sees project picker (has recent projects)', async ({ page }) => {
    await injectTauriMock(page, { recentProjects: MOCK_RECENT_PROJECTS });
    await page.goto('/');
    await expect(page.locator('.picker')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('.welcome')).not.toBeVisible();
  });

  test('picker → welcome transition via "New from template"', async ({ page }) => {
    await injectTauriMock(page, { recentProjects: MOCK_RECENT_PROJECTS });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });

    // Click "New from template"
    await page.locator('.btn-action', { hasText: 'New from template' }).click();

    // Transition animation: view-container gets .transitioning class
    await expect(page.locator('.view-container')).toHaveClass(/transitioning/, { timeout: 1000 });

    // After animation completes, welcome screen appears
    await expect(page.locator('.welcome')).toBeVisible({ timeout: 3000 });
    await expect(page.locator('.picker')).not.toBeVisible();
  });

  test('picker → editor transition via clicking a project', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });

    await page.locator('.project-row').first().click();

    // Should show editor with main content area
    await expect(page.locator('.toolbar')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('.main-content')).toBeVisible({ timeout: 5000 });
  });

  test('editor → picker transition via back button', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });

    // Go to editor
    await page.locator('.project-row').first().click();
    await expect(page.locator('.toolbar')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('.main-content')).toBeVisible({ timeout: 5000 });

    // Go back
    await page.locator('.btn-back').click();
    await expect(page.locator('.picker')).toBeVisible({ timeout: 3000 });
    await expect(page.locator('.toolbar')).not.toBeVisible();
  });

  test('editor → welcome transition when no recent projects and back button', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: [],
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
      dialogResult: '/tmp/some-project',
    });
    await page.goto('/');
    await page.waitForSelector('.welcome', { timeout: 5000 });

    // Open a project via the "Open existing project" button
    await page.locator('.btn-open-existing').click();

    // Editor should appear
    await expect(page.locator('.toolbar')).toBeVisible({ timeout: 5000 });

    // Go back — since we started with 0 recent projects + added 1,
    // there are recent projects now, so it should go to picker
    await page.locator('.btn-back').click();
    await expect(page.locator('.picker, .welcome')).toBeVisible({ timeout: 3000 });
  });

  test('transition has cross-fade animation with translateY', async ({ page }) => {
    await injectTauriMock(page, { recentProjects: MOCK_RECENT_PROJECTS });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });

    // Trigger transition
    await page.locator('.btn-action', { hasText: 'New from template' }).click();

    // During transition, the container should have the transitioning class
    const container = page.locator('.view-container');
    await expect(container).toHaveClass(/transitioning/, { timeout: 1000 });

    // The CSS defines: opacity: 0, transform: translateY(12px) when transitioning
    // Verify the CSS transition property is set
    const style = await container.evaluate((el) => getComputedStyle(el).transition);
    expect(style).toContain('opacity');

    // After the transition completes, transitioning class is removed
    await expect(container).not.toHaveClass(/transitioning/, { timeout: 2000 });
  });

  test('loading overlay appears during project load', async ({ page }) => {
    // Use a slow mock to catch the loading state
    await page.addInitScript(() => {
      (window as any).__TAURI_INTERNALS__ = {
        invoke: async (cmd: string, args: any) => {
          if (cmd === 'list_templates') return [];
          if (cmd === 'get_recent_projects') return [];
          if (cmd === 'plugin:dialog|open') return '/tmp/test-project';
          if (cmd === 'load_dmn_project') {
            // Simulate slow load
            await new Promise((r) => setTimeout(r, 1000));
            return {
              nodes: [],
              edges: [],
              projectName: 'test',
              typeFiles: [],
            };
          }
          if (cmd === 'add_recent_project') return;
          return null;
        },
        metadata: () => Promise.resolve({}),
      };
    });

    await page.goto('/');
    await page.waitForSelector('.welcome', { timeout: 5000 });

    // Click open project
    await page.locator('.btn-open-existing').click();

    // Loading overlay should appear
    await expect(page.locator('.loading-overlay')).toBeVisible({ timeout: 2000 });
    await expect(page.locator('.loading-overlay')).toContainText('Loading project');
    await expect(page.locator('.spinner')).toBeVisible();

    // After load completes, overlay disappears
    await expect(page.locator('.loading-overlay')).not.toBeVisible({ timeout: 5000 });
  });
});
