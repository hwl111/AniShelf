import { useQuery } from "@tanstack/react-query";
import { Database, FolderOpen, RefreshCw, ShieldCheck } from "lucide-react";

import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/utils";

import { getDatabaseStatus } from "../api";

interface StatusRowProps {
  label: string;
  value: string;
  tone?: "default" | "success" | "danger";
}

function StatusRow({ label, value, tone = "default" }: StatusRowProps) {
  return (
    <div className="flex items-center justify-between gap-3 text-sm">
      <span className="text-muted-foreground">{label}</span>
      <span
        className={cn(
          "font-medium",
          tone === "success" && "text-emerald-400",
          tone === "danger" && "text-destructive",
        )}
      >
        {value}
      </span>
    </div>
  );
}

export function SettingsPage() {
  const databaseStatus = useQuery({
    queryKey: ["database-status"],
    queryFn: getDatabaseStatus,
  });

  const status = databaseStatus.data;

  return (
    <PageContainer>
      <PageHeader
        title="设置"
        description="主题、数据与备份设置"
        actions={
          <Button
            variant="outline"
            size="sm"
            onClick={() => void databaseStatus.refetch()}
            disabled={databaseStatus.isFetching}
          >
            <RefreshCw
              className={cn("size-4", databaseStatus.isFetching && "animate-spin")}
              aria-hidden="true"
            />
            刷新
          </Button>
        }
      />

      {databaseStatus.isPending ? <LoadingState label="正在读取数据库状态…" /> : null}

      {databaseStatus.isError ? (
        <ErrorState
          title="无法读取数据库状态"
          description={
            databaseStatus.error instanceof Error ? databaseStatus.error.message : "未知错误"
          }
          onRetry={() => void databaseStatus.refetch()}
        />
      ) : null}

      {status ? (
        <div className="grid gap-4 md:grid-cols-2">
          <Card className="bg-card/40">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-sm">
                <Database className="size-4" aria-hidden="true" />
                数据库文件
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <p className="font-mono text-xs break-all text-muted-foreground">{status.path}</p>
              <StatusRow label="文件大小" value={formatBytes(status.sizeBytes)} />
            </CardContent>
          </Card>

          <Card className="bg-card/40">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-sm">
                <ShieldCheck className="size-4" aria-hidden="true" />
                连接设置
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2">
              <StatusRow
                label="外键约束"
                value={status.foreignKeys ? "已启用" : "未启用"}
                tone={status.foreignKeys ? "success" : "danger"}
              />
              <StatusRow label="日志模式" value={status.journalMode.toUpperCase()} />
              <StatusRow label="已应用迁移" value={`${status.migrationCount} 个`} />
            </CardContent>
          </Card>

          <Card className="bg-card/40 md:col-span-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-sm">
                <FolderOpen className="size-4" aria-hidden="true" />
                备份提示
              </CardTitle>
            </CardHeader>
            <CardContent className="text-sm text-muted-foreground">
              数据库、封面和附件位于应用数据目录；原始书籍文件默认保留在原位置，备份数据时需要单独备份这些书籍文件。
            </CardContent>
          </Card>
        </div>
      ) : null}
    </PageContainer>
  );
}
