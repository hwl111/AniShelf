import { Search } from "lucide-react";
import { useEffect, useState } from "react";
import { Link, useSearchParams } from "react-router";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

import { useSearch } from "../queries";
import type { SearchEntityType, SearchResult } from "../types";

const entityLabels: Record<SearchEntityType, string> = {
  anime: "番剧",
  book: "书籍",
  note: "笔记",
  annotation: "批注",
};

const entityOrder: SearchEntityType[] = ["anime", "book", "note", "annotation"];

type TypeFilter = "all" | SearchEntityType;

function useDebouncedValue(value: string, delay: number): string {
  const [debounced, setDebounced] = useState(value);

  useEffect(() => {
    const timer = window.setTimeout(() => setDebounced(value), delay);
    return () => window.clearTimeout(timer);
  }, [value, delay]);

  return debounced;
}

function resultHref(result: SearchResult): string | null {
  switch (result.entityType) {
    case "anime":
      return `/anime/${result.id}`;
    case "book":
      return `/bookshelf/${result.id}`;
    case "note":
      return `/notes/${result.id}`;
    case "annotation":
      return result.parentId ? `/bookshelf/${result.parentId}` : null;
  }
}

function ResultRow({ result }: { result: SearchResult }) {
  const href = resultHref(result);
  const content = (
    <>
      <div className="flex min-w-0 flex-wrap items-center gap-2">
        <Badge variant="outline">{entityLabels[result.entityType]}</Badge>
        <span className="truncate text-sm font-medium">{result.title}</span>
      </div>
      {result.subtitle ? (
        <p className="mt-1 line-clamp-2 text-sm text-muted-foreground">{result.subtitle}</p>
      ) : null}
    </>
  );

  if (!href) {
    return <div className="rounded-lg border border-border/70 px-3 py-2.5">{content}</div>;
  }

  return (
    <Link
      to={href}
      className="block rounded-lg border border-border/70 px-3 py-2.5 transition-colors hover:border-ring/40 hover:bg-muted/40"
    >
      {content}
    </Link>
  );
}

export function SearchPage() {
  const [searchParams] = useSearchParams();
  const [input, setInput] = useState(() => searchParams.get("q") ?? "");
  const [typeFilter, setTypeFilter] = useState<TypeFilter>("all");
  const query = useDebouncedValue(input.trim(), 250);
  const searchQuery = useSearch(query);
  const results = searchQuery.data ?? [];

  const counts = results.reduce<Record<SearchEntityType, number>>(
    (accumulator, result) => {
      accumulator[result.entityType] += 1;
      return accumulator;
    },
    { anime: 0, book: 0, note: 0, annotation: 0 },
  );

  const filtered =
    typeFilter === "all" ? results : results.filter((result) => result.entityType === typeFilter);

  return (
    <PageContainer>
      <PageHeader title="搜索" description="统一检索番剧、书籍、笔记与批注" />

      <Card className="bg-card/40">
        <CardContent className="space-y-3 py-4">
          <div className="relative">
            <Search
              className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground"
              aria-hidden="true"
            />
            <Input
              className="pl-9"
              value={input}
              placeholder="输入关键词，例如：芙莉莲、读书笔记"
              aria-label="搜索关键词"
              onChange={(event) => setInput(event.target.value)}
            />
          </div>

          <div className="flex flex-wrap gap-2">
            <Button
              size="sm"
              variant={typeFilter === "all" ? "default" : "outline"}
              onClick={() => setTypeFilter("all")}
            >
              全部 {results.length}
            </Button>
            {entityOrder.map((type) => (
              <Button
                key={type}
                size="sm"
                variant={typeFilter === type ? "default" : "outline"}
                onClick={() => setTypeFilter(type)}
              >
                {entityLabels[type]} {counts[type]}
              </Button>
            ))}
          </div>

          <p className="text-xs text-muted-foreground">
            中文搜索：3 个字符及以上使用全文索引，1–2 个字符使用模糊回退；每类最多显示 20 条。
          </p>
        </CardContent>
      </Card>

      {query.length === 0 ? (
        <EmptyState
          icon={Search}
          title="输入关键词开始搜索"
          description="可搜索番剧标题/别名/简介、书名/作者、笔记标题/正文与批注内容。"
        />
      ) : null}

      {query.length > 0 && searchQuery.isPending ? <LoadingState label="正在搜索…" /> : null}

      {query.length > 0 && searchQuery.isError ? (
        <ErrorState
          title="搜索失败"
          description={searchQuery.error.message}
          onRetry={() => void searchQuery.refetch()}
        />
      ) : null}

      {query.length > 0 && !searchQuery.isPending && !searchQuery.isError ? (
        <div className={cn("space-y-5", searchQuery.isFetching && "opacity-80")}>
          {filtered.length === 0 ? (
            <EmptyState
              icon={Search}
              title="没有找到匹配内容"
              description="换一个关键词，或尝试更短/更长的搜索词。"
            />
          ) : null}

          {entityOrder.map((type) => {
            const group = filtered.filter((result) => result.entityType === type);
            if (group.length === 0) {
              return null;
            }

            return (
              <section key={type} className="space-y-2">
                <h2 className="text-sm font-medium text-muted-foreground">
                  {entityLabels[type]} · {group.length}
                </h2>
                <div className="space-y-2">
                  {group.map((result) => (
                    <ResultRow key={`${result.entityType}-${result.id}`} result={result} />
                  ))}
                </div>
              </section>
            );
          })}
        </div>
      ) : null}
    </PageContainer>
  );
}
