import { BookOpen } from "lucide-react";
import { Link, useParams } from "react-router";

import { EmptyState } from "@/components/common/empty-state";
import { ErrorState } from "@/components/common/error-state";
import { LoadingState } from "@/components/common/loading-state";
import { Button } from "@/components/ui/button";
import { useBook } from "@/features/books/queries";

import { EpubReaderPage } from "./epub-reader-page";
import { PdfReaderPage } from "./pdf-reader-page";

export function BookReaderPage() {
  const params = useParams();
  const bookId = Number(params.bookId);
  const validId = Number.isInteger(bookId) && bookId > 0;
  const bookQuery = useBook(validId ? bookId : undefined);

  if (!validId) {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center px-6">
        <div className="w-full max-w-lg">
          <ErrorState title="阅读器无法打开" description="无效的书籍 ID。" />
        </div>
      </div>
    );
  }

  if (bookQuery.isPending) {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center">
        <LoadingState label="正在读取书籍信息…" />
      </div>
    );
  }

  if (bookQuery.isError || !bookQuery.data) {
    return (
      <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center px-6">
        <div className="w-full max-w-lg">
          <ErrorState
            title="无法读取书籍"
            description={bookQuery.error?.message ?? "书籍不存在或已被删除。"}
            onRetry={() => void bookQuery.refetch()}
          />
        </div>
      </div>
    );
  }

  if (bookQuery.data.format === "epub") {
    return <EpubReaderPage />;
  }

  if (bookQuery.data.format === "pdf") {
    return <PdfReaderPage />;
  }

  return (
    <div className="flex h-[calc(100vh-3.5rem)] items-center justify-center px-6">
      <div className="w-full max-w-lg">
        <EmptyState
          icon={BookOpen}
          title="该格式阅读器尚未接入"
          description={`${bookQuery.data.format} 阅读器将在后续阶段提供。`}
          action={
            <Button asChild>
              <Link to={`/bookshelf/${bookQuery.data.id}`}>返回书籍详情</Link>
            </Button>
          }
        />
      </div>
    </div>
  );
}
