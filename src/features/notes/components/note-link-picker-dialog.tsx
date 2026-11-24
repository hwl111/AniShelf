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

import { useAddNoteLink, useNoteLinkCandidates } from "../queries";

interface NoteLinkPickerDialogProps {
  noteId: number;
  existingNoteIds: ReadonlySet<number>;
  onClose: () => void;
}

export function NoteLinkPickerDialog({
  noteId,
  existingNoteIds,
  onClose,
}: NoteLinkPickerDialogProps) {
  const [query, setQuery] = useState("");
  const candidatesQuery = useNoteLinkCandidates(noteId, query.trim(), true);
  const addMutation = useAddNoteLink();

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
          <DialogTitle>添加笔记链接</DialogTitle>
          <DialogDescription>
            被链接的笔记会在其「反向链接」中显示本篇；对方也会收到反向引用。
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-1">
          <Label htmlFor="note-link-search">搜索笔记</Label>
          <Input
            id="note-link-search"
            placeholder="按标题或正文搜索"
            value={query}
            onChange={(event) => setQuery(event.target.value)}
          />
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
            <p className="py-6 text-center text-sm text-muted-foreground">没有找到可链接的笔记。</p>
          ) : null}

          {candidates.map((candidate) => {
            const linked = existingNoteIds.has(candidate.id);
            return (
              <div
                key={candidate.id}
                className="flex items-center justify-between gap-3 rounded-lg border border-border/70 px-3 py-2"
              >
                <div className="min-w-0">
                  <p className="truncate text-sm font-medium">{candidate.title || "无标题笔记"}</p>
                  <p className="truncate text-xs text-muted-foreground">
                    {candidate.excerpt || "（无正文）"}
                  </p>
                </div>
                <Button
                  size="sm"
                  variant="outline"
                  disabled={linked || addMutation.isPending}
                  onClick={() =>
                    addMutation.mutate({ sourceNoteId: noteId, targetNoteId: candidate.id })
                  }
                >
                  {linked ? "已链接" : "链接"}
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
