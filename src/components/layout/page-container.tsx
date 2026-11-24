import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

interface PageContainerProps {
  children: ReactNode;
  className?: string;
}

export function PageContainer({ children, className }: PageContainerProps) {
  return (
    <div
      className={cn("mx-auto flex w-full max-w-6xl flex-col gap-6 px-6 py-6 lg:px-8", className)}
    >
      {children}
    </div>
  );
}
