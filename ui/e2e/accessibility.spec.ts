import { test, expect, MOCK_RECENT_PROJECTS } from './fixtures';

test.describe('Accessibility', () => {
  test('welcome screen template cards are focusable buttons', async ({ welcomePage: page }) => {
    const cards = page.locator('.template-card');
    const count = await cards.count();
    expect(count).toBeGreaterThan(0);

    for (let i = 0; i < count; i++) {
      const card = cards.nth(i);
      const tag = await card.evaluate((el) => el.tagName.toLowerCase());
      expect(tag).toBe('button');
    }
  });

  test('welcome screen "Open existing project" is a focusable button', async ({ welcomePage: page }) => {
    const btn = page.locator('.btn-open-existing');
    const tag = await btn.evaluate((el) => el.tagName.toLowerCase());
    expect(tag).toBe('button');
  });

  test('picker project list uses listbox/option ARIA roles', async ({ pickerPage: page }) => {
    await expect(page.locator('[role="listbox"]')).toBeVisible();

    const options = page.locator('[role="option"]');
    await expect(options).toHaveCount(MOCK_RECENT_PROJECTS.length);

    // First option should be selected
    await expect(options.first()).toHaveAttribute('aria-selected', 'true');
    await expect(options.nth(1)).toHaveAttribute('aria-selected', 'false');
  });

  test('picker arrow-key navigation updates aria-selected', async ({ pickerPage: page }) => {
    const options = page.locator('[role="option"]');

    await page.keyboard.press('ArrowDown');
    await expect(options.first()).toHaveAttribute('aria-selected', 'false');
    await expect(options.nth(1)).toHaveAttribute('aria-selected', 'true');

    await page.keyboard.press('ArrowDown');
    await expect(options.nth(2)).toHaveAttribute('aria-selected', 'true');

    await page.keyboard.press('ArrowUp');
    await expect(options.nth(1)).toHaveAttribute('aria-selected', 'true');
  });

  test('editor back button has title attribute', async ({ page }) => {
    const { injectTauriMock, MOCK_RECENT_PROJECTS, MOCK_FLOW_GRAPH_EMPTY } = await import('./fixtures');
    await injectTauriMock(page, {
      recentProjects: MOCK_RECENT_PROJECTS,
      flowGraph: MOCK_FLOW_GRAPH_EMPTY,
    });
    await page.goto('/');
    await page.waitForSelector('.picker', { timeout: 5000 });
    await page.locator('.project-row').first().click();
    await expect(page.locator('.toolbar')).toBeVisible({ timeout: 5000 });

    await expect(page.locator('.btn-back')).toHaveAttribute('title', 'Back to home');
  });

  test('remove button has title attribute', async ({ pickerPage: page }) => {
    const removeBtn = page.locator('.btn-remove').first();
    await expect(removeBtn).toHaveAttribute('title', 'Remove from recent');
  });

  test('remove button has button role', async ({ pickerPage: page }) => {
    const removeBtn = page.locator('.btn-remove').first();
    await expect(removeBtn).toHaveAttribute('role', 'button');
  });
});
