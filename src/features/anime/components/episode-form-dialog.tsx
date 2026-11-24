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

import type { Episode, EpisodeInput } from "../types";
import { fromDateTimeLocal, toDateTimeLocal } from "./date-time";

interface EpisodeFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  episode?: Episode | null;
  defaultEpisodeNumber: number;
  pending: boolean;
  onSubmit: (input: EpisodeInput) => void;
}

export function EpisodeFormDialog({
  open,
  onOpenChange,
  episode,
  defaultEpisodeNumber,
  pending,
  onSubmit,
}: EpisodeFormDialogProps) {
  const [episodeNumber, setEpisodeNumber] = useState(
    (episode?.episodeNumber ?? defaultEpisodeNumber).toString(),
  );
  const [title, setTitle] = useState(episode?.title ?? "");
  const [airedAt, setAiredAt] = useState(toDateTimeLocal(episode?.airedAt ?? null));

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const parsedNumber = Number(episodeNumber);
    if (!Number.isInteger(parsedNumber) || parsedNumber < 1) {
      return;
    }

    onSubmit({
      episodeNumber: parsedNumber,
      title: title.trim(),
      airedAt: fromDateTimeLocal(airedAt),
    });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <form className="space-y-4" onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{episode ? "编辑单集" : "添加单集"}</DialogTitle>
            <DialogDescription>设置集数与标题；播出时间可选。</DialogDescription>
          </DialogHeader>

          <div className="space-y-3">
            <div className="space-y-1">
              <Label htmlFor="episode-number">集数 *</Label>
              <Input
                id="episode-number"
                type="number"
                required
                min={1}
                max={10000}
                value={episodeNumber}
                onChange={(event) => setEpisodeNumber(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="episode-title">标题</Label>
              <Input
                id="episode-title"
                maxLength={200}
                value={title}
                onChange={(event) => setTitle(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="episode-aired-at">播出时间</Label>
              <Input
                id="episode-aired-at"
                type="datetime-local"
                value={airedAt}
                onChange={(event) => setAiredAt(event.target.value)}
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
