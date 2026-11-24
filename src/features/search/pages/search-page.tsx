import { Search } from "lucide-react";

import { EmptyState } from "@/components/common/empty-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";

export function SearchPage() {
  return (
    <PageContainer>
      <PageHeader title="搜索" description="统一检索番剧、书籍与笔记" />
      <EmptyState
        icon={Search}
        title="暂无可搜索的内容"
        description="添加番剧、书籍和笔记后，可在这里统一检索。"
      />
    </PageContainer>
  );
}
