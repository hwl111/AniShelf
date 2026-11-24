import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";

import { readerThemeColors, readerThemeLabels } from "../preferences";
import type { ReaderPreferences, ReaderTheme } from "../types";

const selectClassName =
  "h-8 w-full rounded-lg border border-input bg-transparent px-2.5 text-sm outline-none transition-colors focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50 dark:bg-input/30";

const FONT_SIZES = [14, 16, 18, 20, 22, 24, 28];
const LINE_HEIGHTS = [1.4, 1.6, 1.7, 1.8, 2.0];
const THEMES: ReaderTheme[] = ["dark", "light", "sepia"];

interface ReaderSettingsDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  preferences: ReaderPreferences;
  onChange: (preferences: ReaderPreferences) => void;
}

export function ReaderSettingsDialog({
  open,
  onOpenChange,
  preferences,
  onChange,
}: ReaderSettingsDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>阅读设置</DialogTitle>
          <DialogDescription>设置会自动保存，并在下次打开时恢复。</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          <div className="space-y-1">
            <Label htmlFor="reader-font-size">字号</Label>
            <select
              id="reader-font-size"
              className={selectClassName}
              value={preferences.fontSize}
              onChange={(event) =>
                onChange({ ...preferences, fontSize: Number(event.target.value) })
              }
            >
              {FONT_SIZES.map((size) => (
                <option key={size} value={size}>
                  {size} px
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1">
            <Label htmlFor="reader-line-height">行距</Label>
            <select
              id="reader-line-height"
              className={selectClassName}
              value={preferences.lineHeight}
              onChange={(event) =>
                onChange({ ...preferences, lineHeight: Number(event.target.value) })
              }
            >
              {LINE_HEIGHTS.map((height) => (
                <option key={height} value={height}>
                  {height.toFixed(1)}
                </option>
              ))}
            </select>
          </div>

          <div className="space-y-1">
            <span className="text-sm font-medium">主题</span>
            <div className="grid grid-cols-3 gap-2">
              {THEMES.map((theme) => {
                const colors = readerThemeColors[theme];
                const selected = preferences.theme === theme;
                return (
                  <button
                    key={theme}
                    type="button"
                    className={cn(
                      "flex flex-col items-center gap-1 rounded-lg border p-2 text-xs transition-colors",
                      selected ? "border-ring bg-muted" : "border-border hover:bg-muted/60",
                    )}
                    onClick={() => onChange({ ...preferences, theme })}
                  >
                    <span
                      className="size-6 rounded-full border border-border"
                      style={{ backgroundColor: colors.background }}
                    />
                    {readerThemeLabels[theme]}
                  </button>
                );
              })}
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            关闭
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
