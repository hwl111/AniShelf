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

import { seasonOptions, watchStatusOptions } from "../constants";
import type { Anime, AnimeInput, Season, WatchStatus } from "../types";
import { selectClassName } from "./field-styles";

function splitList(value: string): string[] {
  return value
    .split(/[,，、\n]/)
    .map((item) => item.trim())
    .filter(Boolean);
}

interface AnimeFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  anime?: Anime | null;
  pending: boolean;
  onSubmit: (input: AnimeInput) => void;
}

export function AnimeFormDialog({
  open,
  onOpenChange,
  anime,
  pending,
  onSubmit,
}: AnimeFormDialogProps) {
  const [title, setTitle] = useState(anime?.title ?? "");
  const [originalTitle, setOriginalTitle] = useState(anime?.originalTitle ?? "");
  const [aliases, setAliases] = useState(anime?.aliases.join("、") ?? "");
  const [summary, setSummary] = useState(anime?.summary ?? "");
  const [year, setYear] = useState(anime?.year?.toString() ?? "");
  const [season, setSeason] = useState<Season | "">(anime?.season ?? "");
  const [totalEpisodes, setTotalEpisodes] = useState(anime?.totalEpisodes?.toString() ?? "");
  const [watchStatus, setWatchStatus] = useState<WatchStatus>(anime?.watchStatus ?? "wish");
  const [rating, setRating] = useState(anime?.rating?.toString() ?? "");
  const [review, setReview] = useState(anime?.review ?? "");
  const [tags, setTags] = useState(anime?.tags.map((tag) => tag.name).join("、") ?? "");

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const parsedYear = year.trim() ? Number(year) : null;
    const parsedTotalEpisodes = totalEpisodes.trim() ? Number(totalEpisodes) : null;
    const parsedRating = rating ? Number(rating) : null;

    onSubmit({
      title: title.trim(),
      originalTitle: originalTitle.trim() || null,
      aliases: splitList(aliases),
      summary: summary.trim(),
      year: parsedYear !== null && Number.isFinite(parsedYear) ? parsedYear : null,
      season: season === "" ? null : season,
      totalEpisodes:
        parsedTotalEpisodes !== null && Number.isFinite(parsedTotalEpisodes)
          ? parsedTotalEpisodes
          : null,
      watchStatus,
      rating: parsedRating !== null && Number.isFinite(parsedRating) ? parsedRating : null,
      review: review.trim(),
      tags: splitList(tags),
    });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
        <form className="space-y-4" onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{anime ? "编辑番剧" : "添加番剧"}</DialogTitle>
            <DialogDescription>
              别名与标签可用逗号、顿号或换行分隔。外部元数据后续可通过 Bangumi 刷新。
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-3 sm:grid-cols-2">
            <div className="space-y-1 sm:col-span-2">
              <Label htmlFor="anime-title">标题 *</Label>
              <Input
                id="anime-title"
                required
                maxLength={300}
                value={title}
                onChange={(event) => setTitle(event.target.value)}
              />
            </div>

            <div className="space-y-1 sm:col-span-2">
              <Label htmlFor="anime-original-title">原名</Label>
              <Input
                id="anime-original-title"
                value={originalTitle}
                onChange={(event) => setOriginalTitle(event.target.value)}
              />
            </div>

            <div className="space-y-1 sm:col-span-2">
              <Label htmlFor="anime-aliases">别名</Label>
              <Input
                id="anime-aliases"
                placeholder="例如：芙莉莲、葬送"
                value={aliases}
                onChange={(event) => setAliases(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="anime-year">年份</Label>
              <Input
                id="anime-year"
                type="number"
                min={1900}
                max={2200}
                placeholder="例如：2023"
                value={year}
                onChange={(event) => setYear(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="anime-season">季度</Label>
              <select
                id="anime-season"
                className={selectClassName}
                value={season}
                onChange={(event) => setSeason(event.target.value as Season | "")}
              >
                <option value="">未指定</option>
                {seasonOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="space-y-1">
              <Label htmlFor="anime-total-episodes">总集数</Label>
              <Input
                id="anime-total-episodes"
                type="number"
                min={0}
                max={10000}
                placeholder="例如：12"
                value={totalEpisodes}
                onChange={(event) => setTotalEpisodes(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="anime-status">观看状态</Label>
              <select
                id="anime-status"
                className={selectClassName}
                value={watchStatus}
                onChange={(event) => setWatchStatus(event.target.value as WatchStatus)}
              >
                {watchStatusOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            <div className="space-y-1">
              <Label htmlFor="anime-rating">评分</Label>
              <select
                id="anime-rating"
                className={selectClassName}
                value={rating}
                onChange={(event) => setRating(event.target.value)}
              >
                <option value="">未评分</option>
                {Array.from({ length: 10 }, (_, index) => index + 1).map((value) => (
                  <option key={value} value={value}>
                    {value} 分
                  </option>
                ))}
              </select>
            </div>

            <div className="space-y-1 sm:col-span-2">
              <Label htmlFor="anime-summary">简介</Label>
              <Textarea
                id="anime-summary"
                rows={3}
                value={summary}
                onChange={(event) => setSummary(event.target.value)}
              />
            </div>

            <div className="space-y-1 sm:col-span-2">
              <Label htmlFor="anime-review">个人评论</Label>
              <Textarea
                id="anime-review"
                rows={3}
                value={review}
                onChange={(event) => setReview(event.target.value)}
              />
            </div>

            <div className="space-y-1 sm:col-span-2">
              <Label htmlFor="anime-tags">标签</Label>
              <Input
                id="anime-tags"
                placeholder="例如：奇幻、治愈"
                value={tags}
                onChange={(event) => setTags(event.target.value)}
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
