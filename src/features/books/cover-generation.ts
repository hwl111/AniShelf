import { getTextBookInfo, readBookBytes, readTextChunk } from "@/features/reader/api";
import { loadPdfDocument } from "@/features/reader/pdf/pdf-renderer";

import { saveGeneratedCover } from "./api";
import type { Book } from "./types";

const COVER_WIDTH = 600;
const COVER_HEIGHT = 800;
const TEXT_SAMPLE_BYTES = 16 * 1024;

const pending = new Map<number, Promise<Book | null>>();
const attempted = new Set<number>();
let queue: Promise<void> = Promise.resolve();

export function ensureBookCover(book: Book): Promise<Book | null> {
  if (book.coverPath || attempted.has(book.id)) {
    return Promise.resolve(null);
  }
  if (book.format !== "pdf" && book.format !== "txt") {
    return Promise.resolve(null);
  }

  const existing = pending.get(book.id);
  if (existing) {
    return existing;
  }

  const task = queue
    .then(async () => {
      const dataUrl =
        book.format === "pdf" ? await renderPdfFirstPage(book) : await renderTextFirstPage(book);
      if (!dataUrl) {
        return null;
      }
      return saveGeneratedCover(book.id, dataUrl);
    })
    .then((updated) => {
      attempted.add(book.id);
      return updated;
    })
    .catch((error: unknown) => {
      attempted.add(book.id);
      throw error;
    })
    .finally(() => {
      pending.delete(book.id);
    });

  pending.set(book.id, task);
  queue = task.then(
    () => undefined,
    () => undefined,
  );
  return task;
}

async function renderPdfFirstPage(book: Book): Promise<string> {
  const data = await readBookBytes(book.id);
  const pdfDocument = await loadPdfDocument(data);

  try {
    if (pdfDocument.numPages < 1) {
      throw new Error("PDF 没有可用的页面");
    }

    const page = await pdfDocument.getPage(1);
    const baseViewport = page.getViewport({ scale: 1 });
    const scale = Math.min(COVER_WIDTH / baseViewport.width, COVER_HEIGHT / baseViewport.height);
    const viewport = page.getViewport({ scale });

    const pageCanvas = document.createElement("canvas");
    pageCanvas.width = Math.max(1, Math.ceil(viewport.width));
    pageCanvas.height = Math.max(1, Math.ceil(viewport.height));
    const pageContext = pageCanvas.getContext("2d");
    if (!pageContext) {
      throw new Error("无法创建 PDF 画布");
    }
    pageContext.fillStyle = "#ffffff";
    pageContext.fillRect(0, 0, pageCanvas.width, pageCanvas.height);
    await page.render({ canvasContext: pageContext, canvas: pageCanvas, viewport }).promise;

    const coverCanvas = createCanvas();
    const context = coverCanvas.getContext("2d");
    if (!context) {
      throw new Error("无法创建封面画布");
    }
    context.fillStyle = "#ffffff";
    context.fillRect(0, 0, COVER_WIDTH, COVER_HEIGHT);
    context.drawImage(
      pageCanvas,
      (COVER_WIDTH - pageCanvas.width) / 2,
      (COVER_HEIGHT - pageCanvas.height) / 2,
    );

    return coverCanvas.toDataURL("image/png");
  } finally {
    await pdfDocument.loadingTask.destroy();
  }
}

async function renderTextFirstPage(book: Book): Promise<string> {
  const info = await getTextBookInfo(book.id);
  const chunk = await readTextChunk(book.id, 0, TEXT_SAMPLE_BYTES, info.encoding);
  const coverCanvas = createCanvas();
  const context = coverCanvas.getContext("2d");
  if (!context) {
    throw new Error("无法创建封面画布");
  }

  const paper = "#f8f4ea";
  const ink = "#1f2937";
  const muted = "#6b7280";
  const accent = "#0f766e";
  const padding = 48;
  const contentWidth = COVER_WIDTH - padding * 2;

  context.fillStyle = paper;
  context.fillRect(0, 0, COVER_WIDTH, COVER_HEIGHT);
  context.strokeStyle = "#d6cfc0";
  context.lineWidth = 2;
  context.strokeRect(20, 20, COVER_WIDTH - 40, COVER_HEIGHT - 40);

  context.textBaseline = "top";
  context.fillStyle = ink;
  context.font = "bold 36px 'Microsoft YaHei', 'Segoe UI', sans-serif";
  let y = padding + 8;
  const titleLines = wrapText(context, book.title || "无标题", contentWidth).slice(0, 2);
  for (const line of titleLines) {
    context.fillText(line, padding, y);
    y += 48;
  }

  if (book.author.trim()) {
    context.fillStyle = muted;
    context.font = "18px 'Microsoft YaHei', 'Segoe UI', sans-serif";
    context.fillText(book.author.trim(), padding, y + 2);
    y += 34;
  }

  context.strokeStyle = accent;
  context.lineWidth = 3;
  context.beginPath();
  context.moveTo(padding, y + 12);
  context.lineTo(padding + 48, y + 12);
  context.stroke();
  y += 44;

  const body = chunk.text.replace(/\r\n?/g, "\n").trimStart();
  context.fillStyle = ink;
  context.font = "19px 'Microsoft YaHei', 'Segoe UI', sans-serif";
  const bodyLimit = COVER_HEIGHT - 84;
  let remaining = body;
  const hasBody = remaining.length > 0;

  if (!hasBody) {
    context.fillStyle = muted;
    context.fillText("（空文件）", padding, y);
  }

  while (remaining.length > 0 && y + 28 <= bodyLimit) {
    const paragraphEnd = remaining.indexOf("\n");
    const paragraph = paragraphEnd === -1 ? remaining : remaining.slice(0, paragraphEnd);
    remaining = paragraphEnd === -1 ? "" : remaining.slice(paragraphEnd + 1);

    if (paragraph.trim() === "") {
      y += 14;
      continue;
    }

    const lines = wrapText(context, paragraph.trim(), contentWidth);
    for (const line of lines) {
      if (y + 28 > bodyLimit) {
        break;
      }
      context.fillText(line, padding, y);
      y += 30;
    }
  }

  context.fillStyle = muted;
  context.font = "14px 'Microsoft YaHei', 'Segoe UI', sans-serif";
  context.fillText("TXT · AniShelf", padding, COVER_HEIGHT - 56);
  context.textAlign = "right";
  context.fillText("1", COVER_WIDTH - padding, COVER_HEIGHT - 56);
  context.textAlign = "left";

  return coverCanvas.toDataURL("image/png");
}

function createCanvas(): HTMLCanvasElement {
  const canvas = document.createElement("canvas");
  canvas.width = COVER_WIDTH;
  canvas.height = COVER_HEIGHT;
  return canvas;
}

function wrapText(context: CanvasRenderingContext2D, text: string, maxWidth: number): string[] {
  const lines: string[] = [];
  let current = "";

  for (const character of text) {
    const candidate = current + character;
    if (current && context.measureText(candidate).width > maxWidth) {
      lines.push(current);
      current = character;
    } else {
      current = candidate;
    }
  }

  if (current) {
    lines.push(current);
  }

  return lines;
}
