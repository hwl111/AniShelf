import { Search } from "lucide-react";
import { useState, type FormEvent } from "react";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { Badge } from "@/components/ui/badge";
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
import { seasonLabels } from "@/features/anime/constants";
import type { Anime } from "@/features/anime/types";

import { useBangumiSearch, useImportBangumi } from "../queries";
import type { BangumiSearchResult } from "../types";

interface BangumiImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImported: (anime: Anime) => void;
}

export function BangumiImportDialog({ open, onOpenChange, onImported }: BangumiImportDialogProps) {
  const [keyword, setKeyword] = useState("");
  const searchMutation = useBangumiSearch();
  const importMutation = useImportBangumi();

  const results = searchMutation.data ?? [];

  function handleSearch(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const value = keyword.trim();
    if (!value) {
      return;
    }
    searchMutation.mutate(value);
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>从 Bangumi 导入</DialogTitle>
          <DialogDescription>
            搜索作品并导入外部元数据。公开接口无需 Token；网络不可用时仍可手动添加番剧。
          </DialogDescription>
        </DialogHeader>

        <form className="flex gap-2" onSubmit={handleSearch}>
          <Input
            placeholder="输入作品名称，例如：芙莉莲"
            value={keyword}
            onChange={(event) => setKeyword(event.target.value)}
          />
          <Button type="submit" disabled={searchMutation.isPending || !keyword.trim()}>
            <Search className="size-4" aria-hidden="true" />
            搜索
          </Button>
        </form>

        <div className="space-y-2">
          {searchMutation.isPending ? <LoadingState label="正在搜索 Bangumi…" /> : null}

          {searchMutation.isError ? (
            <ErrorState
              title="搜索失败"
              description={searchMutation.error.message}
              onRetry={() => searchMutation.mutate(keyword.trim())}
            />
          ) : null}

          {searchMutation.isSuccess && results.length === 0 ? (
            <EmptyState icon={Search} title="没有找到相关作品" description="换一个关键词再试试。" />
          ) : null}

          {results.map((result) => (
            <ResultRow
              key={result.subjectId}
              result={result}
              importing={importMutation.isPending}
              onImport={() =>
                importMutation.mutate(result.subjectId, {
                  onSuccess: (anime) => {
                    onImported(anime);
                    onOpenChange(false);
                  },
                })
              }
            />
          ))}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            关闭
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

interface ResultRowProps {
  result: BangumiSearchResult;
  importing: boolean;
  onImport: () => void;
}

function ResultRow({ result, importing, onImport }: ResultRowProps) {
  return (
    <div className="flex items-start justify-between gap-3 rounded-lg border border-border/60 p-3">
      <div className="min-w-0 space-y-1">
        <p className="font-medium">{result.title}</p>
        {result.originalTitle ? (
          <p className="text-xs text-muted-foreground">{result.originalTitle}</p>
        ) : null}
        <div className="flex flex-wrap items-center gap-1.5">
          {result.year ? (
            <Badge variant="outline">
              {result.year}
              {result.season ? ` · ${seasonLabels[result.season]}` : ""}
            </Badge>
          ) : null}
          {result.totalEpisodes ? <Badge variant="outline">{result.totalEpisodes} 集</Badge> : null}
          {result.airDate ? (
            <span className="text-xs text-muted-foreground">{result.airDate}</span>
          ) : null}
        </div>
        {result.summary ? (
          <p className="line-clamp-2 text-xs text-muted-foreground">{result.summary}</p>
        ) : null}
      </div>
      <Button size="sm" variant="outline" disabled={importing} onClick={onImport}>
        导入
      </Button>
    </div>
  );
}
