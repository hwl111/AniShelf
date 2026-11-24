import type { BookFormat, BookSort } from "./types";

export const bookFormatLabels: Record<BookFormat, string> = {
  epub: "EPUB",
  pdf: "PDF",
  txt: "TXT",
  markdown: "Markdown",
};

export const bookFormatOptions = (Object.keys(bookFormatLabels) as BookFormat[]).map((value) => ({
  value,
  label: bookFormatLabels[value],
}));

export const bookSortLabels: Record<BookSort, string> = {
  updated_desc: "最近更新",
  imported_desc: "最近导入",
  title_asc: "书名 A-Z",
  size_desc: "文件最大",
};

export const bookSortOptions = (Object.keys(bookSortLabels) as BookSort[]).map((value) => ({
  value,
  label: bookSortLabels[value],
}));
