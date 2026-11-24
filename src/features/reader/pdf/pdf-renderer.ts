import * as pdfjsLib from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import type { PDFDocumentProxy, RenderTask } from "pdfjs-dist";

pdfjsLib.GlobalWorkerOptions.workerSrc = workerUrl;

export type PdfZoomMode = "fit-width" | "fit-page" | "custom";

export interface PdfViewLocation {
  page: number;
  mode: PdfZoomMode;
  scale: number;
}

export interface PdfRenderResult {
  scale: number;
  width: number;
  height: number;
}

const ZOOM_MODES: PdfZoomMode[] = ["fit-width", "fit-page", "custom"];

export function loadPdfDocument(data: ArrayBuffer): Promise<PDFDocumentProxy> {
  const base = import.meta.env.BASE_URL;

  return pdfjsLib.getDocument({
    data: new Uint8Array(data),
    cMapUrl: `${base}pdfjs/cmaps/`,
    cMapPacked: true,
    standardFontDataUrl: `${base}pdfjs/standard_fonts/`,
    wasmUrl: `${base}pdfjs/wasm/`,
    iccUrl: `${base}pdfjs/iccs/`,
    useSystemFonts: false,
  }).promise;
}

export function parsePdfLocation(json: string | null | undefined): PdfViewLocation | null {
  if (!json) {
    return null;
  }

  try {
    const value = JSON.parse(json) as Partial<PdfViewLocation>;
    const page =
      typeof value.page === "number" && Number.isInteger(value.page) && value.page > 0
        ? value.page
        : null;
    const mode = ZOOM_MODES.includes(value.mode as PdfZoomMode)
      ? (value.mode as PdfZoomMode)
      : "fit-width";
    const scale =
      typeof value.scale === "number" && Number.isFinite(value.scale)
        ? Math.min(5, Math.max(0.2, value.scale))
        : 1;

    if (page === null) {
      return null;
    }

    return { page, mode, scale };
  } catch {
    return null;
  }
}

export function pdfLocationJson(page: number, mode: PdfZoomMode, scale: number): string {
  return JSON.stringify({ page, mode, scale });
}

export async function renderPdfPage(
  document: PDFDocumentProxy,
  pageNumber: number,
  zoomMode: PdfZoomMode,
  customScale: number,
  container: HTMLElement,
  canvas: HTMLCanvasElement,
  onTask?: (task: RenderTask) => void,
): Promise<PdfRenderResult> {
  const page = await document.getPage(pageNumber);
  const baseViewport = page.getViewport({ scale: 1 });
  const containerWidth = Math.max(1, container.clientWidth - 24);
  const containerHeight = Math.max(1, container.clientHeight - 24);

  let scale = customScale;
  if (zoomMode === "fit-width") {
    scale = containerWidth / baseViewport.width;
  } else if (zoomMode === "fit-page") {
    scale = Math.min(containerWidth / baseViewport.width, containerHeight / baseViewport.height);
  }
  scale = Math.max(0.2, Math.min(5, scale));

  const viewport = page.getViewport({ scale });
  const ratio = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.floor(viewport.width * ratio));
  canvas.height = Math.max(1, Math.floor(viewport.height * ratio));
  canvas.style.width = `${Math.floor(viewport.width)}px`;
  canvas.style.height = `${Math.floor(viewport.height)}px`;

  const context = canvas.getContext("2d", { alpha: false });
  if (!context) {
    throw new Error("无法创建 PDF 画布上下文");
  }

  context.setTransform(ratio, 0, 0, ratio, 0, 0);
  context.save();
  context.fillStyle = "#ffffff";
  context.fillRect(0, 0, viewport.width, viewport.height);
  context.restore();

  const task = page.render({
    canvasContext: context,
    canvas,
    viewport,
  });
  onTask?.(task);
  await task.promise;

  return { scale, width: viewport.width, height: viewport.height };
}
