import { Settings } from "lucide-react";

import { EmptyState } from "@/components/common/empty-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";

export function SettingsPage() {
  return (
    <PageContainer>
      <PageHeader title="设置" description="主题、数据与备份设置" />
      <EmptyState icon={Settings} title="暂无设置项" description="应用设置会在数据层接入后显示。" />
    </PageContainer>
  );
}
