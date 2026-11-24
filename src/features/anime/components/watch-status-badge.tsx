import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

import { watchStatusBadgeClasses, watchStatusLabels } from "../constants";
import type { WatchStatus } from "../types";

interface WatchStatusBadgeProps {
  status: WatchStatus;
  className?: string;
}

export function WatchStatusBadge({ status, className }: WatchStatusBadgeProps) {
  return (
    <Badge variant="outline" className={cn(watchStatusBadgeClasses[status], className)}>
      {watchStatusLabels[status]}
    </Badge>
  );
}
