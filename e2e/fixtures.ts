import { expect, test as base } from "@playwright/test";

/**
 * Test fixture that fails any test producing a browser console error or an
 * uncaught page error. Import `test` and `expect` from here in all E2E specs.
 */
export const test = base.extend<{ failOnConsoleErrors: undefined }>({
  failOnConsoleErrors: [
    async ({ page }, use) => {
      const errors: string[] = [];
      page.on("console", (message) => {
        if (message.type() === "error") errors.push(`console.error: ${message.text()}`);
      });
      page.on("pageerror", (error) => errors.push(`pageerror: ${error.message}`));

      await use(undefined);

      expect(errors, "Unexpected browser errors").toEqual([]);
    },
    { auto: true },
  ],
});

export { expect };
