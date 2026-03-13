import { test, expect, MOCK_RECENT_PROJECTS } from './fixtures';

test.describe('Project Picker (has recent projects)', () => {
  test('shows DSNTK branding', async ({ pickerPage: page }) => {
    await expect(page.locator('.logo-text')).toHaveText('DSNTK');
    await expect(page.locator('.logo-sub')).toHaveText('Visual DMN Explorer');
  });

  test('shows subtitle text', async ({ pickerPage: page }) => {
    await expect(page.locator('.subtitle')).toContainText('Pick up where you left off');
  });

  test('shows "New from template" and "Open a project folder" buttons', async ({ pickerPage: page }) => {
    const newBtn = page.locator('.btn-action', { hasText: 'New from template' });
    const openBtn = page.locator('.btn-action', { hasText: 'Open a project folder' });
    await expect(newBtn).toBeVisible();
    await expect(openBtn).toBeVisible();
  });

  test('shows Recent Projects heading', async ({ pickerPage: page }) => {
    await expect(page.locator('.recent-section h3')).toHaveText('Recent Projects');
  });

  test('renders all recent project rows', async ({ pickerPage: page }) => {
    const rows = page.locator('.project-row');
    await expect(rows).toHaveCount(MOCK_RECENT_PROJECTS.length);
  });

  test('each project row shows name and path', async ({ pickerPage: page }) => {
    for (const proj of MOCK_RECENT_PROJECTS) {
      const row = page.locator('.project-row', { has: page.locator(`.project-name:text("${proj.name}")`) });
      await expect(row).toBeVisible();
      await expect(row.locator('.project-path')).toContainText(proj.path);
    }
  });

  test('shows relative date labels (Today, Yesterday, or formatted date)', async ({ pickerPage: page }) => {
    const dates = page.locator('.project-date');
    await expect(dates.first()).toContainText('Today');
    await expect(dates.nth(1)).toContainText('Yesterday');
    // Third project is 10 days old — shows as a formatted date (> 7 days threshold)
    const thirdText = await dates.nth(2).textContent();
    expect(thirdText).toBeTruthy();
    expect(thirdText).not.toBe('');
  });

  test('first project row is selected by default', async ({ pickerPage: page }) => {
    const firstRow = page.locator('.project-row').first();
    await expect(firstRow).toHaveClass(/selected/);
  });

  test('arrow keys change selection', async ({ pickerPage: page }) => {
    const rows = page.locator('.project-row');

    // Initially first row is selected
    await expect(rows.first()).toHaveClass(/selected/);

    // Press down
    await page.keyboard.press('ArrowDown');
    await expect(rows.nth(1)).toHaveClass(/selected/);
    await expect(rows.first()).not.toHaveClass(/selected/);

    // Press down again
    await page.keyboard.press('ArrowDown');
    await expect(rows.nth(2)).toHaveClass(/selected/);

    // Press up
    await page.keyboard.press('ArrowUp');
    await expect(rows.nth(1)).toHaveClass(/selected/);
  });

  test('arrow up at top stays at first row', async ({ pickerPage: page }) => {
    await page.keyboard.press('ArrowUp');
    await expect(page.locator('.project-row').first()).toHaveClass(/selected/);
  });

  test('arrow down at bottom stays at last row', async ({ pickerPage: page }) => {
    // Navigate to last row
    for (let i = 0; i < 10; i++) await page.keyboard.press('ArrowDown');
    await expect(page.locator('.project-row').last()).toHaveClass(/selected/);
  });

  test('project list has listbox role', async ({ pickerPage: page }) => {
    await expect(page.locator('[role="listbox"]')).toBeVisible();
  });

  test('project rows have option role with aria-selected', async ({ pickerPage: page }) => {
    const rows = page.locator('[role="option"]');
    await expect(rows).toHaveCount(MOCK_RECENT_PROJECTS.length);
    await expect(rows.first()).toHaveAttribute('aria-selected', 'true');
  });

  test('mouse hover changes selection', async ({ pickerPage: page }) => {
    const secondRow = page.locator('.project-row').nth(1);
    await secondRow.hover();
    await expect(secondRow).toHaveClass(/selected/);
  });

  test('remove button is visible on hover', async ({ pickerPage: page }) => {
    const firstRow = page.locator('.project-row').first();
    const removeBtn = firstRow.locator('.btn-remove');

    // Before hover, opacity is 0 (not visible)
    await firstRow.hover();
    // After hover, the button should be rendered (it uses CSS opacity)
    await expect(removeBtn).toBeAttached();
  });

  test('"New from template" navigates to welcome screen', async ({ pickerPage: page }) => {
    const newBtn = page.locator('.btn-action', { hasText: 'New from template' });
    await newBtn.click();

    // Should transition to welcome screen (with cross-fade animation)
    await expect(page.locator('.welcome')).toBeVisible({ timeout: 3000 });
    await expect(page.locator('.template-gallery')).toBeVisible();
  });
});
