import type { TocEntry } from "../epub/adapter";

interface TocPanelProps {
  entries: TocEntry[];
  onSelect: (href: string) => void;
}

export function TocPanel({ entries, onSelect }: TocPanelProps) {
  return (
    <aside className="w-64 shrink-0 overflow-y-auto border-r border-border bg-card/20 p-3">
      <h2 className="mb-2 text-xs font-medium text-muted-foreground">目录</h2>

      {entries.length === 0 ? (
        <p className="text-xs text-muted-foreground">此 EPUB 没有可用目录。</p>
      ) : (
        <div className="space-y-0.5">
          {entries.map((entry) => (
            <button
              key={`${entry.id}-${entry.href}`}
              type="button"
              title={entry.label}
              className="block w-full truncate rounded-md py-1.5 pr-2 text-left text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
              style={{ paddingLeft: `${8 + entry.depth * 12}px` }}
              onClick={() => onSelect(entry.href)}
            >
              {entry.label}
            </button>
          ))}
        </div>
      )}
    </aside>
  );
}
