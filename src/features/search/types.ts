export type SearchEntityType = "anime" | "book" | "note" | "annotation";

export interface SearchResult {
  entityType: SearchEntityType;
  id: number;
  title: string;
  subtitle: string;
  parentId: number | null;
}
