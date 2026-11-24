import { Activity, BookOpen, StickyNote, Tv } from "lucide-react";

import { EmptyState } from "@/components/common/empty-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

const overviewSections = [
  { title: "正在追番", emptyText: "暂无正在观看的番剧", icon: Tv },
  { title: "最近阅读", emptyText: "暂无阅读记录", icon: BookOpen },
  { title: "最近笔记", emptyText: "暂无最近编辑的笔记", icon: StickyNote },
] as const;

export function DashboardPage() {
  return (
    <PageContainer>
      <PageHeader title="首页" description="追番、阅读与笔记的整体概览" />

      <div className="grid gap-4 md:grid-cols-3">
        {overviewSections.map((section) => (
          <Card key={section.title} className="bg-card/40">
            <CardHeader>
              <CardTitle className="text-sm">{section.title}</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-col items-center gap-2 py-8 text-center text-muted-foreground">
                <section.icon className="size-5" aria-hidden="true" />
                <p className="text-xs">{section.emptyText}</p>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <EmptyState
        icon={Activity}
        title="还没有活动记录"
        description="观看番剧、阅读书籍或编辑笔记后，这里会显示最近的活动时间线。"
      />
    </PageContainer>
  );
}
