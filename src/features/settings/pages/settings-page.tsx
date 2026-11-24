import { useQuery } from "@tanstack/react-query";
import { Cloud, Database, RefreshCw, ShieldCheck } from "lucide-react";
import { useState } from "react";

import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  useBangumiTokenStatus,
  useCheckBangumiConnection,
  useSetBangumiToken,
} from "@/features/bangumi/queries";
import { formatBytes } from "@/lib/format";
import { cn } from "@/lib/utils";

import { getDatabaseStatus } from "../api";
import { AboutCard } from "../components/about-card";
import { BackupSection } from "../components/backup-section";

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

  const [tokenInput, setTokenInput] = useState("");
  const tokenStatus = useBangumiTokenStatus();
  const setToken = useSetBangumiToken();
  const checkConnection = useCheckBangumiConnection();

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

          <BackupSection />

          <Card className="bg-card/40 md:col-span-2">
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-sm">
                <Cloud className="size-4" aria-hidden="true" />
                Bangumi
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="flex items-center justify-between gap-3 text-sm">
                <span className="text-muted-foreground">访问令牌</span>
                <span
                  className={cn(
                    "font-medium",
                    tokenStatus.data?.hasToken ? "text-emerald-400" : "text-muted-foreground",
                  )}
                >
                  {tokenStatus.data?.hasToken ? "已配置" : "未配置（使用公开接口）"}
                </span>
              </div>

              {tokenStatus.isError ? (
                <p className="text-xs text-destructive">{tokenStatus.error.message}</p>
              ) : null}

              <div className="flex flex-col gap-2 sm:flex-row">
                <Input
                  type="password"
                  placeholder={
                    tokenStatus.data?.hasToken
                      ? "输入新 Token 可替换；留空保存会清除"
                      : "可选：填写 Bangumi Access Token"
                  }
                  value={tokenInput}
                  onChange={(event) => setTokenInput(event.target.value)}
                />
                <Button
                  variant="outline"
                  disabled={setToken.isPending}
                  onClick={() =>
                    setToken.mutate(tokenInput.trim() || null, {
                      onSuccess: () => setTokenInput(""),
                    })
                  }
                >
                  保存 Token
                </Button>
              </div>

              <div className="flex flex-wrap items-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  disabled={checkConnection.isPending}
                  onClick={() => checkConnection.mutate()}
                >
                  {checkConnection.isPending ? "测试中…" : "测试连接"}
                </Button>
                {checkConnection.data ? (
                  <span
                    className={cn(
                      "text-xs",
                      checkConnection.data.ok ? "text-emerald-400" : "text-destructive",
                    )}
                  >
                    {checkConnection.data.message}
                  </span>
                ) : null}
              </div>

              <p className="text-xs text-muted-foreground">
                Token 仅保存在本机数据库，不写入日志或 Git。网络不可用时仍可手动管理番剧与观看记录。
              </p>
            </CardContent>
          </Card>

          <AboutCard />
        </div>
      ) : null}
    </PageContainer>
  );
}
