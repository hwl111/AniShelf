import type { Tag } from "@/features/anime/types";

export type NoteSort = "updated_desc" | "created_desc";
export type NoteRelationType = "anime" | "book" | "episode";

export interface NoteSummary {
  id: number;
  title: string;
  excerpt: string;
  createdAt: number;
  updatedAt: number;
  tags: Tag[];
  isFavorite: boolean;
}

export interface NoteLink {
  noteId: number;
  title: string;
  updatedAt: number;
}

export interface NoteRelation {
  id: number;
  targetType: NoteRelationType;
  targetId: number;
  title: string;
  subtitle: string;
}

export interface Note {
  id: number;
  title: string;
  content: string;
  createdAt: number;
  updatedAt: number;
  tags: Tag[];
  isFavorite: boolean;
  relations: NoteRelation[];
  links: NoteLink[];
  backlinks: NoteLink[];
}

export interface NoteInput {
  title: string;
  content: string;
  tags: string[];
}

export interface NoteFilter {
  query?: string;
  tagId?: number;
  favorite?: boolean;
  relationType?: NoteRelationType;
  relationId?: number;
  sort?: NoteSort;
  limit?: number;
  offset?: number;
}

export interface NoteDeleteImpact {
  noteId: number;
  backlinks: NoteLink[];
  relationCount: number;
}

export interface NoteLinkCandidate {
  id: number;
  title: string;
  excerpt: string;
  updatedAt: number;
}

export interface NoteRelationCandidate {
  id: number;
  title: string;
  subtitle: string;
}

export interface NoteRelationImpact {
  count: number;
  notes: NoteLink[];
}
