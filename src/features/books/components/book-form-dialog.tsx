import { useState, type FormEvent } from "react";

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
import { Textarea } from "@/components/ui/textarea";

import type { Book, BookUpdateInput } from "../types";

function splitList(value: string): string[] {
  return value
    .split(/[,，、\n]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

interface BookFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  book: Book;
  pending: boolean;
  onSubmit: (input: BookUpdateInput) => void;
}

export function BookFormDialog({
  open,
  onOpenChange,
  book,
  pending,
  onSubmit,
}: BookFormDialogProps) {
  const [title, setTitle] = useState(book.title);
  const [author, setAuthor] = useState(book.author);
  const [category, setCategory] = useState(book.category);
  const [tags, setTags] = useState(book.tags.map((tag) => tag.name).join("、"));
  const [description, setDescription] = useState(book.description);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    onSubmit({
      title: title.trim(),
      author: author.trim(),
      description: description.trim(),
      category: category.trim(),
      tags: splitList(tags),
    });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-xl">
        <form className="space-y-4" onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>编辑书籍信息</DialogTitle>
            <DialogDescription>标签可用逗号、顿号或换行分隔。</DialogDescription>
          </DialogHeader>

          <div className="space-y-3">
            <div className="space-y-1">
              <Label htmlFor="book-title">书名 *</Label>
              <Input
                id="book-title"
                required
                maxLength={300}
                value={title}
                onChange={(event) => setTitle(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="book-author">作者</Label>
              <Input
                id="book-author"
                maxLength={200}
                value={author}
                onChange={(event) => setAuthor(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="book-category">分类</Label>
              <Input
                id="book-category"
                maxLength={100}
                placeholder="例如：技术、小说"
                value={category}
                onChange={(event) => setCategory(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="book-tags">标签</Label>
              <Input
                id="book-tags"
                placeholder="例如：Rust、计算机"
                value={tags}
                onChange={(event) => setTags(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="book-description">简介</Label>
              <Textarea
                id="book-description"
                rows={4}
                value={description}
                onChange={(event) => setDescription(event.target.value)}
              />
            </div>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              取消
            </Button>
            <Button type="submit" disabled={pending}>
              {pending ? "保存中…" : "保存"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
