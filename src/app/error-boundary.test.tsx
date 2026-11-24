import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import { AppErrorBoundary } from "./error-boundary";

function Explode(): ReactNode {
  throw new Error("测试错误");
}

describe("AppErrorBoundary", () => {
  it("shows a readable fallback instead of crashing", () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);

    render(
      <AppErrorBoundary>
        <Explode />
      </AppErrorBoundary>,
    );

    expect(screen.getByRole("heading", { name: "界面出现异常" })).toBeInTheDocument();
    expect(screen.getByText("测试错误")).toBeInTheDocument();
  });
});
