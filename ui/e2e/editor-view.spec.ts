import { test as base, expect } from '@playwright/test';
import { injectTauriMock, MOCK_TEMPLATES, MOCK_RECENT_PROJECTS, MOCK_FLOW_GRAPH, MOCK_FLOW_GRAPH_EMPTY, MOCK_DECISION_TABLE } from './fixtures';

const test = base;

/** Navigate from picker to editor by clicking first project row. */
async function navigateToEditor(page: import('@playwright/test').Page) {
  await page.goto('/');
  await page.waitForSelector('.picker', { timeout: 5000 });
  await page.locator('.project-row').first().click();
  await expect(page.locator('.toolbar')).toBeVisible({ timeout: 5000 });
  await expect(page.locator('.main-content')).toBeVisible({ timeout: 5000 });
}

test.describe('Editor View', () => {
  test('clicking a recent project opens the editor', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      templates: MOCK_TEMPLATES,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
      decisionTable: MOCK_DECISION_TABLE,
    });
    await navigateToEditor(page);
    await expect(page.locator('.toolbar .project-name')).toContainText('loan-app');
  });

  test('pressing Enter on selected project opens the editor', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      templates: MOCK_TEMPLATES,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });
    await page.keyboard.press('Enter');
    await expect(page.locator('.toolbar')).toBeVisible({ timeout: 5000 });
    await expect(page.locator('.main-content')).toBeVisible({ timeout: 5000 });
  });

  test('editor toolbar shows DSNTK branding and project name', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      templates: MOCK_TEMPLATES,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await navigateToEditor(page);
    await expect(page.locator('.toolbar .logo')).toHaveText('DSNTK');
    await expect(page.locator('.toolbar .title')).toHaveText('Visual DMN Explorer');
    await expect(page.locator('.toolbar .project-name')).toHaveText('loan-app');
  });

  test('editor toolbar has back button', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      templates: MOCK_TEMPLATES,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await navigateToEditor(page);
    const backBtn = page.locator('.btn-back');
    await expect(backBtn).toBeVisible();
    await expect(backBtn).toHaveAttribute('title', 'Back to home');
  });

  test('back button returns to picker when recent projects exist', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      templates: MOCK_TEMPLATES,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await navigateToEditor(page);
    await page.locator('.btn-back').click();
    await expect(page.locator('.picker')).toBeVisible({ timeout: 3000 });
  });

  test('editor toolbar has "Open Project" button', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      templates: MOCK_TEMPLATES,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await navigateToEditor(page);
    const openBtn = page.locator('.btn-open');
    await expect(openBtn).toBeVisible();
    await expect(openBtn).toHaveText('Open Project');
  });

  test('side panel shows type definitions section', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      templates: MOCK_TEMPLATES,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await navigateToEditor(page);
    await expect(page.locator('.side-panel .types-section h3')).toHaveText('Type Definitions');
    const details = page.locator('.side-panel details');
    await expect(details).toHaveCount(1);
    await expect(details.locator('summary')).toContainText('types/loan.ts');
  });

  test('empty canvas shows placeholder message', async ({ page }) => {
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      templates: MOCK_TEMPLATES,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await navigateToEditor(page);
    await expect(page.locator('.empty-state')).toBeVisible();
    await expect(page.locator('.empty-state h2')).toContainText('Open a Markdown DMN Project');
  });
});
