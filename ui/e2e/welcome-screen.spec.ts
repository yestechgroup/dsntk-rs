import { test, expect, MOCK_TEMPLATES } from './fixtures';

test.describe('Welcome Screen (no recent projects)', () => {
  test('shows DSNTK branding and tagline', async ({ welcomePage: page }) => {
    await expect(page.locator('.logo-text')).toHaveText('DSNTK');
    await expect(page.locator('.logo-sub')).toHaveText('Visual DMN Explorer');
    await expect(page.locator('.tagline')).toContainText('Markdown-native DMN');
  });

  test('shows "Open existing project" button', async ({ welcomePage: page }) => {
    const btn = page.locator('.btn-open-existing');
    await expect(btn).toBeVisible();
    await expect(btn).toContainText('Open existing project');
  });

  test('shows template gallery section', async ({ welcomePage: page }) => {
    await expect(page.locator('.templates-section h2')).toHaveText('Start from a template');
    await expect(page.locator('.templates-hint')).toContainText('Choose a template');
  });

  test('renders all 6 template cards', async ({ welcomePage: page }) => {
    const cards = page.locator('.template-card');
    await expect(cards).toHaveCount(6);
  });

  test('each template card shows name, description, and features', async ({ welcomePage: page }) => {
    for (const tmpl of MOCK_TEMPLATES) {
      const card = page.locator('.template-card', { has: page.locator(`h3:text("${tmpl.name}")`) });
      await expect(card).toBeVisible();
      await expect(card.locator('.card-desc')).toContainText(tmpl.description.slice(0, 20));
      await expect(card.locator('.node-count')).toContainText(`${tmpl.nodeCount} nodes`);

      const tags = card.locator('.feature-tag');
      await expect(tags).toHaveCount(tmpl.features.length);
    }
  });

  test('does not show the editor toolbar', async ({ welcomePage: page }) => {
    await expect(page.locator('.toolbar')).not.toBeVisible();
  });

  test('does not show error bar by default', async ({ welcomePage: page }) => {
    await expect(page.locator('.error-bar')).not.toBeVisible();
  });
});
