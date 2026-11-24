import { Library } from "lucide-react";

import { EmptyState } from "@/components/common/empty-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";

export function BookshelfPage() {
  return (
    <PageContainer>
      <PageHeader title="书架" description="管理本地电子书与阅读进度" />
      <EmptyState
        icon={Library}
        title="书架还是空的"
        description="导入 EPUB、PDF、TXT 或 Markdown 文件后，会显示在这里。"
      />
    </PageContainer>
  );
}
