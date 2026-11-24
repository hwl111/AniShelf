import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter } from "react-router";
import { RouterProvider } from "react-router/dom";
import { describe, expect, it } from "vitest";

import { AppProviders } from "./providers";
import { routes } from "./router";

function renderAt(path: string) {
  const router = createMemoryRouter(routes, { initialEntries: [path] });
  render(
    <AppProviders>
      <RouterProvider router={router} />
    </AppProviders>,
  );
}

describe("app routing", () => {
  it("renders the dashboard and sidebar at the root route", () => {
    renderAt("/");

    expect(screen.getByRole("heading", { name: "首页" })).toBeInTheDocument();
    expect(screen.getByRole("navigation", { name: "主导航" })).toBeInTheDocument();
  });

  it("navigates from the sidebar to the anime page", async () => {
    const user = userEvent.setup();
    renderAt("/");

    await user.click(screen.getByRole("link", { name: "番剧" }));

    expect(await screen.findByRole("heading", { name: "番剧" })).toBeInTheDocument();
    expect(screen.getByText("还没有番剧")).toBeInTheDocument();
  });

  it("renders the not-found page for unknown routes", () => {
    renderAt("/missing");

    expect(screen.getByRole("heading", { name: "页面不存在" })).toBeInTheDocument();
  });
});
