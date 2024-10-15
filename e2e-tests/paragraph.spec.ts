import { test, expect } from '@playwright/test';

test('has title', async ({ page }) => {
  await page.goto('/examples/tests/paragraph/paragraph');

  // Expect a title "to contain" a substring.
  await expect(page).toHaveTitle(/Stencila/);
});



