import { TriangleAlert } from "lucide-react";
import { Component, type ErrorInfo, type ReactNode } from "react";

import { Button } from "@/components/ui/button";

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  message: string | null;
}

export class AppErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false, message: null };

  static getDerivedStateFromError(error: unknown): ErrorBoundaryState {
    return {
      hasError: true,
      message: error instanceof Error ? error.message : String(error),
    };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("[AniShelf] Unhandled UI error", error, info.componentStack);
  }

  private handleReload = () => {
    window.location.reload();
  };

  render() {
    if (!this.state.hasError) {
      return this.props.children;
    }

    return (
      <div className="flex min-h-screen items-center justify-center bg-background px-6 text-foreground">
        <div className="w-full max-w-md space-y-4 rounded-xl border border-border bg-card p-6 text-center">
          <TriangleAlert className="mx-auto size-10 text-destructive" aria-hidden="true" />
          <h1 className="text-lg font-semibold">界面出现异常</h1>
          <p className="text-sm text-muted-foreground">
            应用遇到了未处理的错误。你的本地数据没有被修改。请尝试重新加载；若问题持续，请查看日志。
          </p>
          {this.state.message ? (
            <pre className="max-h-32 overflow-auto rounded-md bg-muted p-3 text-left text-xs whitespace-pre-wrap text-muted-foreground">
              {this.state.message}
            </pre>
          ) : null}
          <Button onClick={this.handleReload}>重新加载</Button>
        </div>
      </div>
    );
  }
}
