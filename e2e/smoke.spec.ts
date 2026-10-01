import { expect, test } from "./fixtures";

test.describe("application shell", () => {
  test("opens on the dashboard", async ({ page }) => {
    await page.goto("/");

    await expect(page).toHaveTitle("Visão geral · Planalysis");
    await expect(page.getByRole("heading", { level: 1, name: "Visão geral" })).toBeVisible();
  });

  test("uses the dark theme by default", async ({ page }) => {
    await page.goto("/");

    const colorScheme = await page.evaluate(
      () => getComputedStyle(document.documentElement).colorScheme,
    );
    expect(colorScheme).toBe("dark");
  });

  test("reveals a usable skip link on keyboard focus", async ({ page }) => {
    await page.goto("/");
    await page.keyboard.press("Tab");

    const skipLink = page.getByRole("link", { name: "Pular para o conteúdo" });
    await expect(skipLink).toBeFocused();
    await expect(skipLink).toBeVisible();
    // Regression: `not-sr-only` resets padding, which left a cramped, overlapping box.
    const box = await skipLink.boundingBox();
    expect(box?.height ?? 0).toBeGreaterThanOrEqual(36);

    await page.keyboard.press("Enter");
    await expect(page.locator("main")).toBeFocused();
  });

  test("shows a not-found page for unknown deep links", async ({ page }) => {
    await page.goto("/nao-existe");

    await expect(
      page.getByRole("heading", { level: 1, name: "Página não encontrada" }),
    ).toBeVisible();
    await page.getByRole("link", { name: "Voltar para a visão geral" }).click();
    await expect(page).toHaveURL("/");
  });
});

test.describe("desktop navigation", { tag: "@desktop" }, () => {
  test("navigates with the persistent sidebar", async ({ page }) => {
    await page.goto("/");

    const sidebar = page.getByRole("navigation", { name: "Navegação principal" });
    await expect(sidebar).toBeVisible();
    await expect(page.getByRole("button", { name: "Abrir menu" })).toBeHidden();

    await sidebar.getByRole("link", { name: "Transações" }).click();
    await expect(page).toHaveURL("/transacoes");
    await expect(page.getByRole("heading", { level: 1, name: "Transações" })).toBeVisible();
    await expect(sidebar.getByRole("link", { name: "Transações" })).toHaveAttribute(
      "aria-current",
      "page",
    );
  });
});

test.describe("mobile navigation", { tag: "@mobile" }, () => {
  test("navigates with the drawer", async ({ page }) => {
    await page.goto("/");

    await expect(page.getByRole("navigation", { name: "Navegação principal" })).toBeHidden();

    await page.getByRole("button", { name: "Abrir menu" }).click();
    const drawer = page.getByRole("dialog");
    await expect(drawer).toBeVisible();

    await drawer.getByRole("link", { name: "Metas" }).click();
    await expect(page).toHaveURL("/metas");
    await expect(page.getByRole("heading", { level: 1, name: "Metas" })).toBeVisible();
    await expect(drawer).toBeHidden();
  });

  test("has no horizontal page overflow", async ({ page }) => {
    await page.goto("/");

    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBe(0);
  });
});
