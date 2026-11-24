import type { NoteRelationType, NoteSort } from "./types";

export const noteRelationTypes: NoteRelationType[] = ["anime", "book", "episode"];

export const noteRelationTypeLabels: Record<NoteRelationType, string> = {
  anime: "番剧",
  book: "书籍",
  episode: "集数",
};

export const noteSortOptions: { value: NoteSort; label: string }[] = [
  { value: "updated_desc", label: "最近更新" },
  { value: "created_desc", label: "最近创建" },
];

export function isNoteRelationType(value: string | null): value is NoteRelationType {
  return value !== null && noteRelationTypes.includes(value as NoteRelationType);
}
