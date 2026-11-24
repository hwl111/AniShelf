import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";

import * as api from "./api";
import type { NoteFilter, NoteInput, NoteRelationType } from "./types";

export const noteKeys = {
  all: ["notes"] as const,
  list: (filter: NoteFilter) => [...noteKeys.all, "list", filter] as const,
  detail: (id: number) => [...noteKeys.all, "detail", id] as const,
  linkCandidates: (noteId: number, query: string) =>
    [...noteKeys.all, "link-candidates", noteId, query] as const,
  relationCandidates: (targetType: NoteRelationType, query: string) =>
    [...noteKeys.all, "relation-candidates", targetType, query] as const,
  deleteImpact: (id: number) => [...noteKeys.all, "delete-impact", id] as const,
  relationImpact: (targetType: NoteRelationType, targetId: number) =>
    [...noteKeys.all, "relation-impact", targetType, targetId] as const,
};

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : "操作失败，请重试";
}

export function useNotes(filter: NoteFilter) {
  return useQuery({
    queryKey: noteKeys.list(filter),
    queryFn: () => api.listNotes(filter),
  });
}

export function useNote(id: number | undefined) {
  return useQuery({
    queryKey: noteKeys.detail(id ?? 0),
    queryFn: () => api.getNote(id as number),
    enabled: id !== undefined,
  });
}

export function useNoteDeleteImpact(id: number | undefined, enabled: boolean) {
  return useQuery({
    queryKey: noteKeys.deleteImpact(id ?? 0),
    queryFn: () => api.getNoteDeleteImpact(id as number),
    enabled: enabled && id !== undefined,
    staleTime: 0,
    refetchOnMount: "always",
  });
}

export function useNoteRelationImpact(
  targetType: NoteRelationType,
  targetId: number,
  enabled: boolean,
) {
  return useQuery({
    queryKey: noteKeys.relationImpact(targetType, targetId),
    queryFn: () => api.getNoteRelationImpact(targetType, targetId),
    enabled,
    staleTime: 0,
    refetchOnMount: "always",
  });
}

export function useNoteLinkCandidates(noteId: number, query: string, enabled: boolean) {
  return useQuery({
    queryKey: noteKeys.linkCandidates(noteId, query),
    queryFn: () => api.listNoteLinkCandidates(noteId, query),
    enabled,
    staleTime: 10_000,
  });
}

export function useNoteRelationCandidates(
  targetType: NoteRelationType,
  query: string,
  enabled: boolean,
) {
  return useQuery({
    queryKey: noteKeys.relationCandidates(targetType, query),
    queryFn: () => api.listNoteRelationCandidates(targetType, query),
    enabled,
    staleTime: 10_000,
  });
}

export function useCreateNote() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: NoteInput) => api.createNote(input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: noteKeys.all });
      void queryClient.invalidateQueries({ queryKey: ["tags"] });
      toast.success("笔记已创建");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useUpdateNote() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, input }: { id: number; input: NoteInput }) => api.updateNote(id, input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: noteKeys.all });
      void queryClient.invalidateQueries({ queryKey: ["tags"] });
      toast.success("笔记已保存");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useDeleteNote() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, force }: { id: number; force: boolean }) => api.deleteNote(id, force),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: noteKeys.all });
      toast.success("笔记已删除");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useAddNoteLink() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ sourceNoteId, targetNoteId }: { sourceNoteId: number; targetNoteId: number }) =>
      api.addNoteLink(sourceNoteId, targetNoteId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: noteKeys.all });
      toast.success("已添加笔记链接");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useRemoveNoteLink() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ sourceNoteId, targetNoteId }: { sourceNoteId: number; targetNoteId: number }) =>
      api.removeNoteLink(sourceNoteId, targetNoteId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: noteKeys.all });
      toast.success("已移除笔记链接");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useAddNoteRelation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      noteId,
      targetType,
      targetId,
    }: {
      noteId: number;
      targetType: NoteRelationType;
      targetId: number;
    }) => api.addNoteRelation(noteId, targetType, targetId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: noteKeys.all });
      toast.success("已添加关联");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}

export function useRemoveNoteRelation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ noteId, relationId }: { noteId: number; relationId: number }) =>
      api.removeNoteRelation(noteId, relationId),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: noteKeys.all });
      toast.success("已移除关联");
    },
    onError: (error) => toast.error(errorMessage(error)),
  });
}
