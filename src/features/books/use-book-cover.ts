import { useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";

import { ensureBookCover } from "./cover-generation";
import { bookKeys } from "./queries";
import type { Book } from "./types";

export function useEnsureBookCover(book: Book | undefined) {
  const queryClient = useQueryClient();

  useEffect(() => {
    if (!book || book.coverPath) {
      return;
    }
    if (book.format !== "pdf" && book.format !== "txt") {
      return;
    }

    let cancelled = false;
    void ensureBookCover(book)
      .then((updated) => {
        if (!cancelled && updated) {
          void queryClient.invalidateQueries({ queryKey: bookKeys.all });
        }
      })
      .catch((error: unknown) => {
        console.error(`[AniShelf] 生成《${book.title}》封面失败`, error);
      });

    return () => {
      cancelled = true;
    };
  }, [book, queryClient]);
}
