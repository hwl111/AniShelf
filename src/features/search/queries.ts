import { keepPreviousData, useQuery } from "@tanstack/react-query";

import * as api from "./api";

export const searchKeys = {
  all: ["search"] as const,
  query: (query: string) => [...searchKeys.all, query] as const,
};

export function useSearch(query: string) {
  const trimmed = query.trim();

  return useQuery({
    queryKey: searchKeys.query(trimmed),
    queryFn: () => api.searchAll(trimmed),
    enabled: trimmed.length > 0,
    staleTime: 10_000,
    placeholderData: keepPreviousData,
  });
}
