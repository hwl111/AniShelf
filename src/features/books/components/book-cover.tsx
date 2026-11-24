import { convertFileSrc } from "@tauri-apps/api/core";
import { BookOpen } from "lucide-react";
import { useState } from "react";

import { cn } from "@/lib/utils";

import type { Book } from "../types";
import { useEnsureBookCover } from "../use-book-cover";

interface BookCoverProps {
  book?: Book;
  coverUrl: string | null;
  title: string;
  className?: string;
}

function CoverImage({
  coverUrl,
  title,
  className,
}: {
  coverUrl: string;
  title: string;
  className?: string;
}) {
  const [failed, setFailed] = useState(false);

  if (failed) {
    return (
      <div
        className={cn("flex items-center justify-center bg-muted text-muted-foreground", className)}
      >
        <BookOpen className="size-8" aria-hidden="true" />
      </div>
    );
  }

  return (
    <img
      src={convertFileSrc(coverUrl)}
      alt={`${title} 封面`}
      className={cn("object-cover", className)}
      onError={() => setFailed(true)}
    />
  );
}

export function BookCover({ book, coverUrl, title, className }: BookCoverProps) {
  useEnsureBookCover(book);

  if (!coverUrl) {
    return (
      <div
        className={cn("flex items-center justify-center bg-muted text-muted-foreground", className)}
      >
        <BookOpen className="size-8" aria-hidden="true" />
      </div>
    );
  }

  return <CoverImage key={coverUrl} coverUrl={coverUrl} title={title} className={className} />;
}
