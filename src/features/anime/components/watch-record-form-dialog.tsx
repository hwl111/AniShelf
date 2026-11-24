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

import type { WatchRecord } from "../types";
import { fromDateTimeLocal, toDateTimeLocal } from "./date-time";

export interface WatchRecordFormValues {
  episodeNumber: number;
  watchedAt: number | null;
  comment: string;
}

interface WatchRecordFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  episodeNumber: number;
  defaultWatchedAt: number;
  record?: WatchRecord | null;
  pending: boolean;
  onSubmit: (values: WatchRecordFormValues) => void;
}

export function WatchRecordFormDialog({
  open,
  onOpenChange,
  episodeNumber,
  defaultWatchedAt,
  record,
  pending,
  onSubmit,
}: WatchRecordFormDialogProps) {
  const [number, setNumber] = useState((record?.episodeNumber ?? episodeNumber).toString());
  const [watchedAt, setWatchedAt] = useState(
    toDateTimeLocal(record?.watchedAt ?? defaultWatchedAt),
  );
  const [comment, setComment] = useState(record?.comment ?? "");

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const parsedNumber = Number(number);
    if (!Number.isInteger(parsedNumber) || parsedNumber < 1) {
      return;
    }

    onSubmit({
      episodeNumber: parsedNumber,
      watchedAt: fromDateTimeLocal(watchedAt),
      comment: comment.trim(),
    });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <form className="space-y-4" onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{record ? "编辑观看记录" : "添加观看记录"}</DialogTitle>
            <DialogDescription>
              可补录历史观看时间，并记录本集感想。留空的观看时间会使用当前时间。
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3">
            <div className="space-y-1">
              <Label htmlFor="record-episode-number">集数 *</Label>
              <Input
                id="record-episode-number"
                type="number"
                required
                min={1}
                max={10000}
                disabled={Boolean(record)}
                value={number}
                onChange={(event) => setNumber(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="record-watched-at">观看时间</Label>
              <Input
                id="record-watched-at"
                type="datetime-local"
                value={watchedAt}
                onChange={(event) => setWatchedAt(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="record-comment">单集感想</Label>
              <Textarea
                id="record-comment"
                rows={4}
                maxLength={2000}
                value={comment}
                onChange={(event) => setComment(event.target.value)}
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
