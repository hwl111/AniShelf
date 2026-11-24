import { BookOpen } from "lucide-react";
import { Link } from "react-router";

import { EmptyState } from "@/components/common/empty-state";
import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/layout/page-header";
import { Button } from "@/components/ui/button";

export function ReaderPage() {
  return (
    <PageContainer>
      <PageHeader title="阅读器" description="继续阅读或选择一本书开始" />
      <EmptyState
        icon={BookOpen}
        title="未选择书籍"
        description="从书架打开一本书后，会在这里继续阅读。"
        action={
          <Button asChild variant="outline">
            <Link to="/bookshelf">前往书架</Link>
          </Button>
        }
      />
    </PageContainer>
  );
}
