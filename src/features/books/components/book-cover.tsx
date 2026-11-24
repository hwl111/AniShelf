import { convertFileSrc } from "@tauri-apps/api/core";
import { BookOpen } from "lucide-react";
import { useState } from "react";

import { cn } from "@/lib/utils";

interface BookCoverProps {
  coverUrl: string | null;
  title: string;
  className?: string;
}

export function BookCover({ coverUrl, title, className }: BookCoverProps) {
  const [failed, setFailed] = useState(false);

  if (!coverUrl || failed) {
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
