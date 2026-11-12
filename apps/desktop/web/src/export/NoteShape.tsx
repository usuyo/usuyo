import type { ExportView } from "../api/endpoints";

/**
 * 一篇导出去的条目笔记长什么样：分隔线以上是你的，那边改了会读回来；以下每次导出照 usuyo 重写。
 * 照笔记软件里打开的样子摆：文件名在上，正文等宽排。
 */
export function NoteShape({ sample, name }: { sample: ExportView["sample"]; name: string }) {
  return (
    <figure className="m-0 overflow-hidden rounded-card border border-border2 bg-card">
      <figcaption className="flex items-center gap-2 border-b border-border px-4 py-2">
        <span className="mono min-w-0 truncate text-tag text-label">{sample.file}</span>
        <span className="ml-auto shrink-0 text-small text-faint">在 {name} 里打开的样子</span>
      </figcaption>
      <div className="px-4 pt-3 pb-4">
        <p className="m-0 mb-1.5 text-small text-sub">你的：在那边改了，下次导出前读回来</p>
        <pre className="mono m-0 text-tag leading-[20px] whitespace-pre-wrap text-text select-text">{sample.yours}</pre>
        <div className="my-3 flex items-center gap-3" aria-hidden="true">
          <span className="h-px flex-1 border-t border-dashed border-border2" />
          <span className="text-small text-faint">这条线以下每次导出重写</span>
          <span className="h-px flex-1 border-t border-dashed border-border2" />
        </div>
        <pre className="mono m-0 text-tag leading-[20px] whitespace-pre-wrap text-sub select-text">{sample.ours}</pre>
      </div>
    </figure>
  );
}
