import { invokeCommand } from "@/lib/ipc";

import type { SearchResult } from "./types";

export function searchAll(query: string): Promise<SearchResult[]> {
  return invokeCommand<SearchResult[]>("search_all", { query });
}
