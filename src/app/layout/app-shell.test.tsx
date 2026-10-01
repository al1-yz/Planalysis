import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, RouterProvider } from "react-router";
import { describe, expect, it } from "vitest";

import { routes } from "../routes";

function renderApp(initialPath = "/") {
  const router = createMemoryRouter(routes, { initialEntries: [initialPath] });
  render(<RouterProvider router={router} />);
  return { user: userEvent.setup() };
}

// jsdom does not apply the Tailwind breakpoints, so both the desktop sidebar and the
// mobile top bar are present. The desktop sidebar is the only navigation rendered
// until the drawer is opened.
function sidebar() {
  return screen.getByRole("navigation", { name: "Navegação principal" });
}

describe("AppShell", () => {
  it("renders the dashboard inside the shell", () => {
    renderApp();

    expect(screen.getByRole("heading", { level: 1, name: "Visão geral" })).toBeInTheDocument();
    expect(within(sidebar()).getByRole("link", { name: "Visão geral" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(within(sidebar()).getByRole("link", { name: "Transações" })).not.toHaveAttribute(
      "aria-current",
    );
  });

  it("navigates between sections from the sidebar", async () => {
    const { user } = renderApp();

    await user.click(within(sidebar()).getByRole("link", { name: "Transações" }));

    expect(screen.getByRole("heading", { level: 1, name: "Transações" })).toBeInTheDocument();
    expect(within(sidebar()).getByRole("link", { name: "Transações" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(within(sidebar()).getByRole("link", { name: "Visão geral" })).not.toHaveAttribute(
      "aria-current",
    );
  });

  it("opens the mobile drawer, navigates, and closes it", async () => {
    const { user } = renderApp();

    await user.click(screen.getByRole("button", { name: "Abrir menu" }));
    const drawer = screen.getByRole("dialog");
    await user.click(within(drawer).getByRole("link", { name: "Contas" }));

    expect(screen.getByRole("heading", { level: 1, name: "Contas" })).toBeInTheDocument();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("closes the mobile drawer with the close button", async () => {
    const { user } = renderApp();

    await user.click(screen.getByRole("button", { name: "Abrir menu" }));
    await user.click(screen.getByRole("button", { name: "Fechar" }));

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("shows a not-found page for unknown paths", () => {
    renderApp("/nao-existe");

    expect(
      screen.getByRole("heading", { level: 1, name: "Página não encontrada" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Voltar para a visão geral" })).toHaveAttribute(
      "href",
      "/",
    );
  });
});
