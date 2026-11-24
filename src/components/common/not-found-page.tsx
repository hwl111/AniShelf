import { FileQuestion } from "lucide-react";
import { Link } from "react-router";

import { EmptyState } from "@/components/common/empty-state";
import { PageContainer } from "@/components/layout/page-container";
import { Button } from "@/components/ui/button";

export function NotFoundPage() {
  return (
    <PageContainer className="justify-center py-24">
      <EmptyState
        icon={FileQuestion}
        title="页面不存在"
        description="你访问的页面不存在或已被移动。"
        action={
          <Button asChild>
            <Link to="/">返回首页</Link>
          </Button>
        }
      />
    </PageContainer>
  );
}
