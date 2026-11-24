import { Plus, StickyNote } from "lucide-react";
import { useMemo } from "react";
import { Link } from "react-router";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

import { useNotes } from "../queries";
import type { NoteFilter } from "../types";
import { formatNoteTime } from "../utils";

interface RelatedNotesCardProps {
  targetType: "anime" | "book";
  targetId: number;
}

export function RelatedNotesCard({ targetType, targetId }: RelatedNotesCardProps) {
  const filter = useMemo<NoteFilter>(
    () => ({
      relationType: targetType,
      relationId: targetId,
      sort: "updated_desc",
      limit: 5,
    }),
    [targetType, targetId],
  );
  const notesQuery = useNotes(filter);
  const notes = notesQuery.data ?? [];

  const newNoteHref = `/notes/new?relationType=${targetType}&relationId=${targetId}`;
  const allNotesHref = `/notes?relationType=${targetType}&relationId=${targetId}`;

  return (
    <Card className="bg-card/40">
      <CardHeader className="flex flex-row items-center justify-between gap-3">
        <CardTitle className="text-sm">相关笔记</CardTitle>
        <div className="flex items-center gap-2">
          {notes.length > 0 ? (
            <Button asChild variant="ghost" size="sm">
              <Link to={allNotesHref}>查看全部</Link>
            </Button>
          ) : null}
          <Button asChild variant="outline" size="sm">
            <Link to={newNoteHref}>
              <Plus className="size-4" aria-hidden="true" />
              新建笔记
            </Link>
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        {notesQuery.isPending ? (
          <p className="text-sm text-muted-foreground">正在读取相关笔记…</p>
        ) : null}

        {notesQuery.isError ? (
          <p className="text-sm text-destructive">{notesQuery.error.message}</p>
        ) : null}

        {notesQuery.data && notes.length === 0 ? (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <StickyNote className="size-4" aria-hidden="true" />
            <span>还没有关联笔记。新建笔记会自动关联到这里。</span>
          </div>
        ) : null}

        {notes.map((note) => (
          <Link
            key={note.id}
            to={`/notes/${note.id}`}
            className="flex items-center justify-between gap-3 rounded-lg border border-transparent px-2 py-1.5 transition-colors hover:border-border/70 hover:bg-muted/40"
          >
            <span className="truncate text-sm">{note.title || "无标题笔记"}</span>
            <span className="shrink-0 text-xs text-muted-foreground">
              {formatNoteTime(note.updatedAt)}
            </span>
          </Link>
        ))}
      </CardContent>
    </Card>
  );
}
