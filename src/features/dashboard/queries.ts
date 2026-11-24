import { useQuery } from "@tanstack/react-query";

import * as api from "./api";

export const dashboardKeys = {
  all: ["dashboard"] as const,
};

export function useDashboard() {
  return useQuery({
    queryKey: dashboardKeys.all,
    queryFn: api.getDashboard,
    staleTime: 0,
    refetchOnMount: "always",
  });
}
