import { Info } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

import { useAppInfo } from "../queries";

export function AboutCard() {
  const infoQuery = useAppInfo();
  const info = infoQuery.data;

  return (
    <Card className="bg-card/40 md:col-span-2">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-sm">
          <Info className="size-4" aria-hidden="true" />
          关于 AniShelf
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-2 text-sm">
        {infoQuery.isPending ? <p className="text-muted-foreground">正在读取版本信息…</p> : null}
        {infoQuery.isError ? <p className="text-destructive">{infoQuery.error.message}</p> : null}
        {info ? (
          <>
            <div className="flex items-center justify-between gap-3">
              <span className="text-muted-foreground">版本</span>
              <span className="font-medium">v{info.version}</span>
            </div>
            <div className="flex items-center justify-between gap-3">
              <span className="text-muted-foreground">数据库 schema</span>
              <span>{info.schemaVersion}</span>
            </div>
            {[
              ["数据目录", info.dataDir],
              ["数据库", info.databasePath],
              ["备份目录", info.backupsDir],
              ["日志目录", info.logsDir],
            ].map(([label, value]) => (
              <div key={label} className="space-y-1">
                <p className="text-xs text-muted-foreground">{label}</p>
                <p className="font-mono text-xs break-all text-muted-foreground">{value}</p>
              </div>
            ))}
          </>
        ) : null}
      </CardContent>
    </Card>
  );
}
