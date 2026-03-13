import { test as base, expect } from '@playwright/test';
import { injectTauriMock, MOCK_TEMPLATES, MOCK_RECENT_PROJECTS, MOCK_FLOW_GRAPH_EMPTY, MOCK_DECISION_TABLE } from './fixtures';

const test = base;

test.describe('Bridge Integration', () => {
  test('initialize() calls list_templates and get_recent_projects', async ({ page }) => {
    await injectTauriMock(page, { recentProjects: MOCK_RECENT_PROJECTS });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });

    const log = await page.evaluate(() => (window as any).__invokeLog);
    const cmds = log.map((e: any) => e.cmd);
    expect(cmds).toContain('list_templates');
    expect(cmds).toContain('get_recent_projects');
  });

  test('opening a project calls load_dmn_project then add_recent_project', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });

    // Click the first project
    await page.locator('.project-row').first().click();
    await expect(page.locator('.toolbar')).toBeVisible({ timeout: 5000 });

    const log = await page.evaluate(() => (window as any).__invokeLog);
    const cmds = log.map((e: any) => e.cmd);
    expect(cmds).toContain('load_dmn_project');
    expect(cmds).toContain('add_recent_project');

    // Verify the project path was passed correctly
    const loadCall = log.find((e: any) => e.cmd === 'load_dmn_project');
    expect(loadCall.args.projectDir).toBe('/home/user/projects/loan-app');
  });

  test('removing a recent project calls remove_recent_project', async ({ page }) => {
    await injectTauriMock(page, { recentProjects: MOCK_RECENT_PROJECTS });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });

    // Hover over first row to reveal remove button, then click it
    const firstRow = page.locator('.project-row').first();
    await firstRow.hover();
    await firstRow.locator('.btn-remove').click({ force: true });

    // Wait for re-render
    await page.waitForTimeout(300);

    const log = await page.evaluate(() => (window as any).__invokeLog);
    const removeCalls = log.filter((e: any) => e.cmd === 'remove_recent_project');
    expect(removeCalls.length).toBeGreaterThan(0);
    expect(removeCalls[0].args.path).toBe('/home/user/projects/loan-app');
  });

  test('opening dialog calls plugin:dialog|open', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: [],
      dialogResult: '/tmp/test-project',
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await page.goto('/');
    await page.waitForSelector('.welcome', { timeout: 5000 });

    await page.locator('.btn-open-existing').click();
    await expect(page.locator('.toolbar')).toBeVisible({ timeout: 5000 });

    const log = await page.evaluate(() => (window as any).__invokeLog);
    const dialogCalls = log.filter((e: any) => e.cmd === 'plugin:dialog|open');
    expect(dialogCalls.length).toBeGreaterThan(0);
  });

  test('null dialog result does not load a project', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: [],
      dialogResult: null,
    });
    await page.goto('/');
    await page.waitForSelector('.welcome', { timeout: 5000 });

    await page.locator('.btn-open-existing').click();

    // Should still be on welcome (dialog was cancelled)
    await page.waitForTimeout(500);
    await expect(page.locator('.welcome')).toBeVisible();

    const log = await page.evaluate(() => (window as any).__invokeLog);
    const loadCalls = log.filter((e: any) => e.cmd === 'load_dmn_project');
    expect(loadCalls.length).toBe(0);
  });

  test('editor shows project data from mock', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });

    await page.locator('.project-row').first().click();
    await expect(page.locator('.toolbar')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('.main-content')).toBeVisible({ timeout: 5000 });

    // Verify project name appears in toolbar
    await expect(page.locator('.toolbar .project-name')).toHaveText('loan-app');
  });
});
