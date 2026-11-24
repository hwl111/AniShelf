import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen, Upload } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

import { useImportBook } from "../queries";
import type { Book } from "../types";

interface BookImportDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onImported: (book: Book) => void;
}

export function BookImportDialog({
  open: dialogOpen,
  onOpenChange,
  onImported,
}: BookImportDialogProps) {
  const [path, setPath] = useState<string | null>(null);
  const [copyToLibrary, setCopyToLibrary] = useState(false);
  const importMutation = useImportBook();

  async function pickFile() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        title: "选择电子书文件",
        filters: [
          {
            name: "电子书",
            extensions: ["epub", "pdf", "txt", "md", "markdown"],
          },
        ],
      });

      if (typeof selected === "string") {
        setPath(selected);
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "无法打开文件选择器");
    }
  }

  return (
    <Dialog open={dialogOpen} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>导入书籍</DialogTitle>
          <DialogDescription>
            选择 EPUB、PDF、TXT 或 Markdown 文件。原始文件默认只读引用，应用不会修改它。
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <div className="flex flex-col gap-2 sm:flex-row">
            <Button type="button" variant="outline" onClick={() => void pickFile()}>
              <FolderOpen className="size-4" aria-hidden="true" />
              选择文件
            </Button>
            <div className="min-w-0 flex-1 rounded-md border border-border/60 bg-muted/30 px-3 py-2 text-xs break-all text-muted-foreground">
              {path ?? "尚未选择文件"}
            </div>
          </div>

          <label className="flex items-start gap-2 text-sm">
            <input
              type="checkbox"
              className="mt-1 size-4 accent-primary"
              checked={copyToLibrary}
              onChange={(event) => setCopyToLibrary(event.target.checked)}
            />
            <span>
              复制到应用资料库（可选）
              <span className="block text-xs text-muted-foreground">
                原文件保持不变；复制会额外占用磁盘空间，适合原文件可能被移动的情况。
              </span>
            </span>
          </label>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            取消
          </Button>
          <Button
            disabled={!path || importMutation.isPending}
            onClick={() => {
              if (!path) {
                return;
              }
              importMutation.mutate(
                { path, copyToLibrary },
                {
                  onSuccess: (result) => {
                    onImported(result.book);
                    onOpenChange(false);
                  },
                },
              );
            }}
          >
            <Upload className="size-4" aria-hidden="true" />
            {importMutation.isPending ? "导入中…" : "导入"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
