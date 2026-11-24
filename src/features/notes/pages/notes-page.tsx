import { StickyNote } from "lucide-react";

import { EmptyState } from "@/components/common/empty-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";

export function NotesPage() {
  return (
    <PageContainer>
      <PageHeader title="笔记" description="记录想法并关联番剧、书籍与批注" />
      <EmptyState icon={StickyNote} title="还没有笔记" description="创建笔记后，会显示在这里。" />
    </PageContainer>
  );
}
