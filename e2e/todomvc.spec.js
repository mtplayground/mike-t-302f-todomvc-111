const { expect, test } = require('playwright/test');

test('matches the official TodoMVC behavior set', async ({ page }) => {
  await resetTodos(page);
  await page.goto('/');

  const newTodo = page.getByPlaceholder('What needs to be done?');
  const items = page.locator('.todo-list li');

  await expect(page.locator('.main')).toBeHidden();
  await expect(page.locator('.footer')).toBeHidden();

  await newTodo.fill('   ');
  await newTodo.press('Enter');
  await expect(items).toHaveCount(0);
  await expect(newTodo).toHaveValue('');

  await addTodo(newTodo, items, 'buy milk', 1);
  await expect(page.locator('.main')).toBeVisible();
  await expect(page.locator('.footer')).toBeVisible();
  await expect(page.locator('.todo-count')).toHaveText('1 item left');

  await addTodo(newTodo, items, 'write tests', 2);
  await addTodo(newTodo, items, 'ship app', 3);
  await expect(page.locator('.todo-count')).toHaveText('3 items left');
  await expect(items.locator('label')).toHaveText(['buy milk', 'write tests', 'ship app']);

  await items.nth(0).locator('.toggle').check();
  await expect(items.nth(0)).toHaveClass(/completed/);
  await expect(page.locator('.todo-count')).toHaveText('2 items left');
  await expect(page.locator('.clear-completed')).toBeVisible();

  await items.nth(1).locator('label').dblclick();
  const editInput = items.nth(1).locator('.edit');
  await expect(items.nth(1)).toHaveClass(/editing/);
  await editInput.fill('write e2e tests');
  await editInput.press('Enter');
  await expect(items.nth(1).locator('label')).toHaveText('write e2e tests');

  await items.nth(1).locator('label').dblclick();
  await editInput.fill('cancelled edit');
  await editInput.press('Escape');
  await expect(items.nth(1).locator('label')).toHaveText('write e2e tests');

  await filter(page, 'Active', /#\/active$/);
  await expect(items).toHaveCount(2);
  await expect(items.locator('label')).toHaveText(['write e2e tests', 'ship app']);

  await filter(page, 'Completed', /#\/completed$/);
  await expect(items).toHaveCount(1);
  await expect(items.locator('label')).toHaveText(['buy milk']);

  await filter(page, 'All', /#\/$/);
  await expect(items).toHaveCount(3);

  await page.locator('#toggle-all').check({ force: true });
  await expect(page.locator('.todo-list li.completed')).toHaveCount(3);
  await expect(page.locator('.todo-count')).toHaveText('0 items left');

  await page.locator('#toggle-all').uncheck({ force: true });
  await expect(page.locator('.todo-list li.completed')).toHaveCount(0);
  await expect(page.locator('.todo-count')).toHaveText('3 items left');

  await items.nth(0).locator('.toggle').check();
  await page.locator('.clear-completed').click();
  await expect(items).toHaveCount(2);
  await expect(items.locator('label')).toHaveText(['write e2e tests', 'ship app']);
  await expect(page.locator('.clear-completed')).toBeHidden();

  await items.nth(0).locator('label').dblclick();
  await items.nth(0).locator('.edit').fill('    ');
  await items.nth(0).locator('.edit').press('Enter');
  await expect(items).toHaveCount(1);
  await expect(items.locator('label')).toHaveText(['ship app']);

  await items.nth(0).hover();
  await items.nth(0).locator('.destroy').click();
  await expect(items).toHaveCount(0);
  await expect(page.locator('.main')).toBeHidden();
  await expect(page.locator('.footer')).toBeHidden();
});

async function addTodo(newTodo, items, title, expectedCount) {
  await newTodo.fill(title);
  await newTodo.press('Enter');
  await expect(items).toHaveCount(expectedCount);
  await expect(newTodo).toHaveValue('');
}

async function filter(page, name, expectedUrl) {
  await page.locator('.filters a', { hasText: name }).click();
  await expect(page).toHaveURL(expectedUrl);
  await expect(page.locator('.filters a.selected')).toHaveText(name);
}

async function resetTodos(page) {
  const response = await page.request.get('/api/todos');
  expect(response.ok()).toBeTruthy();

  const body = await response.json();
  for (const todo of body.data) {
    const deleted = await page.request.delete(`/api/todos/${todo.id}`);
    expect(deleted.ok()).toBeTruthy();
  }
}
