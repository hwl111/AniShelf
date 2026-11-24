import { Tv } from "lucide-react";

import { EmptyState } from "@/components/common/empty-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";

export function AnimePage() {
  return (
    <PageContainer>
      <PageHeader title="番剧" description="管理追番进度、评分与观看记录" />
      <EmptyState
        icon={Tv}
        title="还没有番剧"
        description="手动添加作品或从 Bangumi 导入后，会显示在这里。"
      />
    </PageContainer>
  );
}
