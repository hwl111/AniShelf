import { isRouteErrorResponse, useRouteError } from "react-router";

import { ErrorState } from "@/components/common/error-state";

export function RouteErrorPage() {
  const error = useRouteError();

  let description = "页面加载失败，请稍后重试。";
  if (isRouteErrorResponse(error)) {
    description = `${error.status} ${error.statusText}`;
  } else if (error instanceof Error) {
    description = error.message;
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-background px-6 text-foreground">
      <div className="w-full max-w-lg">
        <ErrorState
          title="页面加载失败"
          description={description}
          onRetry={() => window.location.reload()}
        />
      </div>
    </div>
  );
}
