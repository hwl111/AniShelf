import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";

import { noteRelationTypeLabels, noteRelationTypes } from "../constants";
import { useAddNoteRelation, useNoteRelationCandidates } from "../queries";
import type { NoteRelationType } from "../types";

const selectClassName =
  "h-8 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm outline-none transition-colors focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 dark:bg-input/30";

interface NoteRelationPickerDialogProps {
  noteId: number;
  existingTargets: ReadonlySet<string>;
  onClose: () => void;
}

function targetKey(targetType: NoteRelationType, targetId: number): string {
  return `${targetType}:${targetId}`;
}

export function NoteRelationPickerDialog({
  noteId,
  existingTargets,
  onClose,
}: NoteRelationPickerDialogProps) {
  const [targetType, setTargetType] = useState<NoteRelationType>("anime");
  const [query, setQuery] = useState("");
  const candidatesQuery = useNoteRelationCandidates(targetType, query.trim(), true);
  const addMutation = useAddNoteRelation();

  const candidates = candidatesQuery.data ?? [];

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open) {
          onClose();
        }
      }}
    >
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>添加关联</DialogTitle>
          <DialogDescription>
            将笔记关联到番剧、书籍或集数；重复关联不会重复添加。
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-3 sm:grid-cols-[120px_minmax(0,1fr)]">
          <div className="space-y-1">
            <Label htmlFor="note-relation-type">类型</Label>
            <select
              id="note-relation-type"
              className={selectClassName}
              value={targetType}
              onChange={(event) => {
                setTargetType(event.target.value as NoteRelationType);
                setQuery("");
              }}
            >
              {noteRelationTypes.map((type) => (
                <option key={type} value={type}>
                  {noteRelationTypeLabels[type]}
                </option>
              ))}
            </select>
          </div>
          <div className="space-y-1">
            <Label htmlFor="note-relation-search">搜索</Label>
            <Input
              id="note-relation-search"
              placeholder={`搜索${noteRelationTypeLabels[targetType]}`}
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
          </div>
        </div>

        <div className="max-h-80 space-y-2 overflow-y-auto pr-1">
          {candidatesQuery.isPending ? (
            <p className="py-6 text-center text-sm text-muted-foreground">正在读取…</p>
          ) : null}

          {candidatesQuery.isError ? (
            <p className="py-6 text-center text-sm text-destructive">
              {candidatesQuery.error.message}
            </p>
          ) : null}

          {candidatesQuery.data && candidates.length === 0 ? (
            <p className="py-6 text-center text-sm text-muted-foreground">
              没有找到可关联的{noteRelationTypeLabels[targetType]}。
            </p>
          ) : null}

          {candidates.map((candidate) => {
            const related = existingTargets.has(targetKey(targetType, candidate.id));
            return (
              <div
                key={candidate.id}
                className={cn(
                  "flex items-center justify-between gap-3 rounded-lg border border-border/70 px-3 py-2",
                  related && "opacity-60",
                )}
              >
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium">{candidate.title}</p>
                  {candidate.subtitle ? (
                    <p className="truncate text-xs text-muted-foreground">{candidate.subtitle}</p>
                  ) : null}
                </div>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={related || addMutation.isPending}
                  onClick={() =>
                    addMutation.mutate({
                      noteId,
                      targetType,
                      targetId: candidate.id,
                    })
                  }
                >
                  {related ? "已关联" : "关联"}
                </Button>
              </div>
            );
          })}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose}>
            完成
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
