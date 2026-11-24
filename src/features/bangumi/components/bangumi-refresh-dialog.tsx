import { CheckCircle2, RefreshCw } from "lucide-react";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import type { Anime } from "@/features/anime/types";

import { useApplyBangumiRefresh, useBangumiRefreshPreview } from "../queries";

interface BangumiRefreshDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  anime: Anime;
}

export function BangumiRefreshDialog({ open, onOpenChange, anime }: BangumiRefreshDialogProps) {
  const preview = useBangumiRefreshPreview(anime.id, open);
  const applyMutation = useApplyBangumiRefresh(anime.id);
  const changes = preview.data?.changes ?? [];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>Bangumi 刷新预览</DialogTitle>
          <DialogDescription>
            只更新标题、原名、别名、简介、封面链接、年份、季度与总集数。观看状态、评分、评论、标签和观看记录不会被覆盖。
          </DialogDescription>
        </DialogHeader>

        {preview.isPending ? <LoadingState label="正在获取 Bangumi 数据…" /> : null}

        {preview.isError ? (
          <ErrorState
            title="无法获取 Bangumi 数据"
            description={preview.error.message}
            onRetry={() => void preview.refetch()}
          />
        ) : null}

        {preview.data && changes.length === 0 ? (
          <EmptyState
            icon={CheckCircle2}
            title="已是最新"
            description="Bangumi 外部元数据与本地一致。"
          />
        ) : null}

        {changes.length > 0 ? (
          <div className="space-y-2">
            {changes.map((change) => (
              <div key={change.field} className="space-y-1 rounded-md border border-border/60 p-3">
                <p className="text-xs font-medium text-muted-foreground">{change.label}</p>
                <p className="text-xs break-all text-muted-foreground line-through">
                  {change.oldValue}
                </p>
                <p className="text-sm break-all">{change.newValue}</p>
              </div>
            ))}
          </div>
        ) : null}

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button
            disabled={applyMutation.isPending || changes.length === 0 || preview.isError}
            onClick={() =>
              applyMutation.mutate(undefined, {
                onSuccess: () => onOpenChange(false),
              })
            }
          >
            <RefreshCw className="size-4" aria-hidden="true" />
            {applyMutation.isPending ? "应用中…" : "应用更新"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
