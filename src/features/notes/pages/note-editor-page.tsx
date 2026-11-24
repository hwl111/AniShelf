import { ArrowLeft, Link2, Plus, Save, Star, Trash2, X } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { Link, useBlocker, useNavigate, useParams, useSearchParams } from "react-router";
import { toast } from "sonner";

import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
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
import { Textarea } from "@/components/ui/textarea";

import { isNoteRelationType } from "../constants";
import { NoteLinkPickerDialog } from "../components/note-link-picker-dialog";
import { NoteRelationPickerDialog } from "../components/note-relation-picker-dialog";
import {
  useAddNoteRelation,
  useCreateNote,
  useDeleteNote,
  useNote,
  useNoteDeleteImpact,
  useRemoveNoteLink,
  useRemoveNoteRelation,
  useSetNoteFavorite,
  useUpdateNote,
} from "../queries";
import type { Note, NoteInput, NoteRelationType } from "../types";
import { formatNoteTime, splitTags } from "../utils";

interface NoteDraft {
  title: string;
  content: string;
  tagsText: string;
}

function draftFromNote(note: Note | null): NoteDraft {
  return {
    title: note?.title ?? "",
    content: note?.content ?? "",
    tagsText: note?.tags.map((tag) => tag.name).join("，") ?? "",
  };
}

export function NoteEditorPage() {
  const params = useParams();
  const isNew = params.noteId === undefined;
  const noteId = Number(params.noteId);
  const validId = isNew || (Number.isInteger(noteId) && noteId > 0);
  const noteQuery = useNote(isNew || !validId ? undefined : noteId);

  if (!isNew && !validId) {
    return (
      <PageContainer>
        <ErrorState title="笔记不存在" description="无效的笔记 ID。" />
      </PageContainer>
    );
  }

  if (!isNew && noteQuery.isPending) {
    return (
      <PageContainer>
        <LoadingState label="正在读取笔记…" />
      </PageContainer>
    );
  }

  if (!isNew && (noteQuery.isError || !noteQuery.data)) {
    return (
      <PageContainer>
        <ErrorState
          title="无法读取笔记"
          description={noteQuery.error?.message ?? "笔记不存在或已被删除。"}
          onRetry={() => void noteQuery.refetch()}
        />
      </PageContainer>
    );
  }

  return <NoteEditor key={isNew ? "new" : noteQuery.data?.id} note={noteQuery.data ?? null} />;
}

interface NoteEditorProps {
  note: Note | null;
}

function NoteEditor({ note }: NoteEditorProps) {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  const initialDraft = useMemo(() => draftFromNote(note), [note]);
  const [title, setTitle] = useState(initialDraft.title);
  const [content, setContent] = useState(initialDraft.content);
  const [tagsText, setTagsText] = useState(initialDraft.tagsText);
  const [baseline, setBaseline] = useState<NoteDraft>(initialDraft);
  const [saving, setSaving] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [relationPickerOpen, setRelationPickerOpen] = useState(false);
  const [linkPickerOpen, setLinkPickerOpen] = useState(false);

  const createMutation = useCreateNote();
  const updateMutation = useUpdateNote();
  const deleteMutation = useDeleteNote();
  const addRelationMutation = useAddNoteRelation();
  const removeRelationMutation = useRemoveNoteRelation();
  const removeLinkMutation = useRemoveNoteLink();
  const favoriteMutation = useSetNoteFavorite();
  const deleteImpactQuery = useNoteDeleteImpact(note?.id, deleteOpen);

  const dirty =
    title !== baseline.title || content !== baseline.content || tagsText !== baseline.tagsText;
  const allowNavigationRef = useRef(false);
  const blocker = useBlocker(() => dirty && !allowNavigationRef.current);

  const relationTypeParam = searchParams.get("relationType");
  const relationIdParam = Number(searchParams.get("relationId"));
  const pendingRelation: { type: NoteRelationType; targetId: number } | null =
    note === null &&
    isNoteRelationType(relationTypeParam) &&
    Number.isInteger(relationIdParam) &&
    relationIdParam > 0
      ? { type: relationTypeParam, targetId: relationIdParam }
      : null;

  const existingRelationKeys = useMemo(
    () =>
      new Set(
        (note?.relations ?? []).map((relation) => `${relation.targetType}:${relation.targetId}`),
      ),
    [note?.relations],
  );
  const existingLinkIds = useMemo(
    () => new Set((note?.links ?? []).map((link) => link.noteId)),
    [note?.links],
  );

  useEffect(() => {
    if (!dirty) {
      return;
    }

    const handler = (event: BeforeUnloadEvent) => {
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", handler);
    return () => window.removeEventListener("beforeunload", handler);
  }, [dirty]);

  async function finishCreate(created: Note, draft: NoteDraft) {
    if (pendingRelation) {
      try {
        await addRelationMutation.mutateAsync({
          noteId: created.id,
          targetType: pendingRelation.type,
          targetId: pendingRelation.targetId,
        });
      } catch (error) {
        console.error("[AniShelf] 添加初始关联失败", error);
      }
    }

    setTitle(draft.title);
    setBaseline(draft);
    setSaving(false);
    allowNavigationRef.current = true;
    navigate(`/notes/${created.id}`, { replace: true });
  }

  function handleSave() {
    if (saving) {
      return;
    }

    const trimmedTitle = title.trim();
    if (!trimmedTitle && !content.trim()) {
      toast.error("笔记标题和正文不能同时为空");
      return;
    }

    const draft: NoteDraft = {
      title: trimmedTitle,
      content,
      tagsText,
    };
    const input: NoteInput = {
      title: trimmedTitle,
      content,
      tags: splitTags(tagsText),
    };

    setSaving(true);
    if (!note) {
      createMutation.mutate(input, {
        onSuccess: (created) => {
          void finishCreate(created, draft);
        },
        onError: () => setSaving(false),
      });
    } else {
      updateMutation.mutate(
        { id: note.id, input },
        {
          onSuccess: () => {
            setTitle(draft.title);
            setBaseline(draft);
            setSaving(false);
          },
          onError: () => setSaving(false),
        },
      );
    }
  }

  return (
    <PageContainer>
      <div>
        <Button asChild variant="ghost" size="sm" className="text-muted-foreground">
          <Link to="/notes">
            <ArrowLeft className="size-4" aria-hidden="true" />
            返回笔记
          </Link>
        </Button>
      </div>

      <PageHeader
        title={note ? "编辑笔记" : "新建笔记"}
        description={
          note
            ? `创建于 ${formatNoteTime(note.createdAt)} · 更新于 ${formatNoteTime(note.updatedAt)}`
            : "标题和正文至少填写一项；保存后会出现在笔记列表中。"
        }
        actions={
          <>
            {note ? (
              <Button
                variant="outline"
                size="sm"
                disabled={favoriteMutation.isPending}
                onClick={() => favoriteMutation.mutate({ id: note.id, favorite: !note.isFavorite })}
              >
                <Star
                  className={note.isFavorite ? "fill-amber-300 text-amber-300" : "size-4"}
                  aria-hidden="true"
                />
                {note.isFavorite ? "已收藏" : "收藏"}
              </Button>
            ) : null}
            {dirty ? <Badge variant="outline">未保存</Badge> : null}
            <Button onClick={handleSave} disabled={saving}>
              <Save className="size-4" aria-hidden="true" />
              {saving ? "保存中…" : "保存"}
            </Button>
          </>
        }
      />

      <div
        className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_320px]"
        onKeyDown={(event) => {
          if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") {
            event.preventDefault();
            handleSave();
          }
        }}
      >
        <Card className="bg-card/40">
          <CardContent className="space-y-4 py-4">
            <div className="space-y-1">
              <Label htmlFor="note-title">标题</Label>
              <Input
                id="note-title"
                value={title}
                placeholder="无标题笔记"
                className="text-lg font-medium"
                onChange={(event) => setTitle(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="note-content">正文</Label>
              <Textarea
                id="note-content"
                value={content}
                placeholder="写下阅读或观看时的想法…"
                className="min-h-[420px] leading-7"
                onChange={(event) => setContent(event.target.value)}
              />
              <p className="text-xs text-muted-foreground">
                纯文本编辑；Ctrl+S 可快速保存，离开页面前会提示未保存的修改。
              </p>
            </div>
          </CardContent>
        </Card>

        <aside className="space-y-4">
          <Card className="bg-card/40">
            <CardHeader>
              <CardTitle className="text-sm">标签</CardTitle>
            </CardHeader>
            <CardContent className="space-y-1">
              <Input
                value={tagsText}
                placeholder="用逗号、顿号或换行分隔"
                aria-label="笔记标签"
                onChange={(event) => setTagsText(event.target.value)}
              />
              <p className="text-xs text-muted-foreground">最多 20 个标签，重复标签会自动合并。</p>
            </CardContent>
          </Card>

          <Card className="bg-card/40">
            <CardHeader className="flex flex-row items-center justify-between gap-2">
              <CardTitle className="text-sm">关联对象</CardTitle>
              <Button
                size="sm"
                variant="outline"
                disabled={!note}
                onClick={() => setRelationPickerOpen(true)}
              >
                <Plus className="size-4" aria-hidden="true" />
                添加
              </Button>
            </CardHeader>
            <CardContent className="space-y-2">
              {!note ? (
                <p className="text-sm text-muted-foreground">保存笔记后可添加关联。</p>
              ) : note.relations.length === 0 ? (
                <p className="text-sm text-muted-foreground">暂无关联。</p>
              ) : (
                note.relations.map((relation) => (
                  <div
                    key={relation.id}
                    className="flex items-start justify-between gap-2 rounded-lg border border-border/70 px-2.5 py-2"
                  >
                    <div className="min-w-0">
                      <p className="truncate text-sm">{relation.title}</p>
                      {relation.subtitle ? (
                        <p className="truncate text-xs text-muted-foreground">
                          {relation.subtitle}
                        </p>
                      ) : null}
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      aria-label="移除关联"
                      title="移除关联"
                      disabled={removeRelationMutation.isPending}
                      onClick={() =>
                        removeRelationMutation.mutate({
                          noteId: note.id,
                          relationId: relation.id,
                        })
                      }
                    >
                      <X className="size-4" aria-hidden="true" />
                    </Button>
                  </div>
                ))
              )}
            </CardContent>
          </Card>

          <Card className="bg-card/40">
            <CardHeader className="flex flex-row items-center justify-between gap-2">
              <CardTitle className="text-sm">链接到其他笔记</CardTitle>
              <Button
                size="sm"
                variant="outline"
                disabled={!note}
                onClick={() => setLinkPickerOpen(true)}
              >
                <Link2 className="size-4" aria-hidden="true" />
                链接
              </Button>
            </CardHeader>
            <CardContent className="space-y-2">
              {!note ? (
                <p className="text-sm text-muted-foreground">保存笔记后可添加链接。</p>
              ) : note.links.length === 0 ? (
                <p className="text-sm text-muted-foreground">暂无出链。</p>
              ) : (
                note.links.map((link) => (
                  <div key={link.noteId} className="flex items-center justify-between gap-2">
                    <Link
                      to={`/notes/${link.noteId}`}
                      className="min-w-0 truncate text-sm text-primary hover:underline"
                    >
                      {link.title}
                    </Link>
                    <Button
                      variant="ghost"
                      size="icon"
                      aria-label="移除链接"
                      title="移除链接"
                      disabled={removeLinkMutation.isPending}
                      onClick={() =>
                        removeLinkMutation.mutate({
                          sourceNoteId: note.id,
                          targetNoteId: link.noteId,
                        })
                      }
                    >
                      <X className="size-4" aria-hidden="true" />
                    </Button>
                  </div>
                ))
              )}
            </CardContent>
          </Card>

          <Card className="bg-card/40">
            <CardHeader>
              <CardTitle className="text-sm">反向链接</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              {!note ? (
                <p className="text-sm text-muted-foreground">
                  保存并保存其他笔记的链接后，这里会显示引用。
                </p>
              ) : note.backlinks.length === 0 ? (
                <p className="text-sm text-muted-foreground">暂无笔记链接到本篇。</p>
              ) : (
                note.backlinks.map((link) => (
                  <Link
                    key={link.noteId}
                    to={`/notes/${link.noteId}`}
                    className="block truncate text-sm text-primary hover:underline"
                  >
                    {link.title}
                  </Link>
                ))
              )}
            </CardContent>
          </Card>

          {note ? (
            <Card className="border-destructive/30 bg-card/40">
              <CardHeader>
                <CardTitle className="text-sm">删除</CardTitle>
              </CardHeader>
              <CardContent>
                <Button variant="destructive" size="sm" onClick={() => setDeleteOpen(true)}>
                  <Trash2 className="size-4" aria-hidden="true" />
                  删除笔记
                </Button>
              </CardContent>
            </Card>
          ) : null}
        </aside>
      </div>

      {note && relationPickerOpen ? (
        <NoteRelationPickerDialog
          noteId={note.id}
          existingTargets={existingRelationKeys}
          onClose={() => setRelationPickerOpen(false)}
        />
      ) : null}

      {note && linkPickerOpen ? (
        <NoteLinkPickerDialog
          noteId={note.id}
          existingNoteIds={existingLinkIds}
          onClose={() => setLinkPickerOpen(false)}
        />
      ) : null}

      <Dialog
        open={blocker.state === "blocked"}
        onOpenChange={(open) => {
          if (!open) {
            blocker.reset?.();
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>有未保存的修改</DialogTitle>
            <DialogDescription>离开此页面会丢失当前编辑内容。要放弃这些修改吗？</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => blocker.reset?.()}>
              留在本页
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                allowNavigationRef.current = true;
                blocker.proceed?.();
              }}
            >
              放弃修改并离开
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {note && deleteOpen ? (
        <Dialog
          open
          onOpenChange={(open) => {
            if (!open) {
              setDeleteOpen(false);
            }
          }}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>删除笔记？</DialogTitle>
              <DialogDescription>
                删除后无法恢复，笔记的标签、链接与关联会一并移除。
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-2 text-sm">
              {deleteImpactQuery.isPending ? (
                <p className="text-muted-foreground">正在检查引用影响…</p>
              ) : null}

              {deleteImpactQuery.isError ? (
                <p className="text-destructive">{deleteImpactQuery.error.message}</p>
              ) : null}

              {deleteImpactQuery.data ? (
                <>
                  {deleteImpactQuery.data.backlinks.length > 0 ? (
                    <div className="space-y-1">
                      <p>
                        以下 {deleteImpactQuery.data.backlinks.length}{" "}
                        篇笔记链接到本笔记，删除后这些链接会被移除：
                      </p>
                      <ul className="list-inside list-disc text-muted-foreground">
                        {deleteImpactQuery.data.backlinks.map((link) => (
                          <li key={link.noteId} className="truncate">
                            {link.title}
                          </li>
                        ))}
                      </ul>
                    </div>
                  ) : (
                    <p className="text-muted-foreground">没有其他笔记链接到本笔记。</p>
                  )}

                  {deleteImpactQuery.data.relationCount > 0 ? (
                    <p className="text-muted-foreground">
                      本篇还关联了 {deleteImpactQuery.data.relationCount}{" "}
                      个番剧/书籍/集数，关联关系会被移除，对象本身不受影响。
                    </p>
                  ) : null}
                </>
              ) : null}
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setDeleteOpen(false)}>
                取消
              </Button>
              <Button
                variant="destructive"
                disabled={deleteMutation.isPending || !deleteImpactQuery.data}
                onClick={() =>
                  deleteMutation.mutate(
                    { id: note.id, force: true },
                    {
                      onSuccess: () => {
                        allowNavigationRef.current = true;
                        navigate("/notes");
                      },
                    },
                  )
                }
              >
                {deleteMutation.isPending ? "删除中…" : "确认删除"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}
    </PageContainer>
  );
}
