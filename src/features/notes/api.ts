import { invokeCommand } from "@/lib/ipc";

import type {
  Note,
  NoteDeleteImpact,
  NoteFilter,
  NoteInput,
  NoteLinkCandidate,
  NoteRelationCandidate,
  NoteRelationImpact,
  NoteRelationType,
  NoteSummary,
} from "./types";

export function listNotes(filter: NoteFilter = {}): Promise<NoteSummary[]> {
  return invokeCommand<NoteSummary[]>("list_notes", { filter });
}

export function getNote(id: number): Promise<Note> {
  return invokeCommand<Note>("get_note", { id });
}

export function createNote(input: NoteInput): Promise<Note> {
  return invokeCommand<Note>("create_note", { input });
}

export function updateNote(id: number, input: NoteInput): Promise<Note> {
  return invokeCommand<Note>("update_note", { id, input });
}

export function deleteNote(id: number, force: boolean): Promise<void> {
  return invokeCommand<void>("delete_note", { id, force });
}

export function setNoteFavorite(id: number, favorite: boolean): Promise<Note> {
  return invokeCommand<Note>("set_note_favorite", { id, favorite });
}

export function getNoteDeleteImpact(id: number): Promise<NoteDeleteImpact> {
  return invokeCommand<NoteDeleteImpact>("get_note_delete_impact", { id });
}

export function getNoteRelationImpact(
  targetType: NoteRelationType,
  targetId: number,
): Promise<NoteRelationImpact> {
  return invokeCommand<NoteRelationImpact>("get_note_relation_impact", {
    targetType,
    targetId,
  });
}

export function listNoteLinkCandidates(
  noteId: number,
  query: string,
): Promise<NoteLinkCandidate[]> {
  return invokeCommand<NoteLinkCandidate[]>("list_note_link_candidates", { noteId, query });
}

export function addNoteLink(sourceNoteId: number, targetNoteId: number): Promise<void> {
  return invokeCommand<void>("add_note_link", { sourceNoteId, targetNoteId });
}

export function removeNoteLink(sourceNoteId: number, targetNoteId: number): Promise<void> {
  return invokeCommand<void>("remove_note_link", { sourceNoteId, targetNoteId });
}

export function listNoteRelationCandidates(
  targetType: NoteRelationType,
  query: string,
): Promise<NoteRelationCandidate[]> {
  return invokeCommand<NoteRelationCandidate[]>("list_note_relation_candidates", {
    targetType,
    query,
  });
}

export function addNoteRelation(
  noteId: number,
  targetType: NoteRelationType,
  targetId: number,
): Promise<void> {
  return invokeCommand<void>("add_note_relation", { noteId, targetType, targetId });
}

export function removeNoteRelation(noteId: number, relationId: number): Promise<void> {
  return invokeCommand<void>("remove_note_relation", { noteId, relationId });
}
