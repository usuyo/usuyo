import { Link } from "@tanstack/react-router";
import { FileUp } from "lucide-react";
import { useState } from "react";
import { DropZone, FileTrigger, Radio, RadioGroup, Text } from "react-aria-components";
import { isNotServed } from "../api/client";
import type { FileReader } from "../api/endpoints";
import { useFileReaders, useImportFile } from "../api/hooks";
import { ErrorLine, NotServedLine } from "../components/ErrorLine";
import { sizeText } from "../item/faces";
import { Button } from "../ui/Button";
import { acceptOf } from "./words";

/**
 * 别的工具导出的文件：挑一个读它的导入插件，选文件（或拖进来），读一遍。读的是一份副本，原文件不动；
 * 读出来的和别的导入一样先给你看，你点了才收。
 */
export function FileImport({ onStarted }: { onStarted: (id: string) => void }) {
  const readers = useFileReaders();
  const send = useImportFile();
  const [readerId, setReaderId] = useState<string | undefined>(undefined);
  const [file, setFile] = useState<File | null>(null);
  const list = readers.data ?? [];
  const reader = list.find((r) => r.id === readerId) ?? list[0];
  const pick = (files: FileList | File[] | null) => {
    const f = files?.[0];
    if (f) {
      setFile(f);
      send.reset();
    }
  };

  return (
    <section aria-labelledby="file-import-h">
      <h2 id="file-import-h" className="m-0 text-section font-semibold text-text">
        别的工具导出的文件
      </h2>
      <p className="m-0 mt-1 text-small text-sub">平台官方给的归档、书签工具和阅读器导出的清单。读的是一份副本，原文件不动；先读出来给你看，你点了才收。</p>
      {isNotServed(readers.error) ? (
        <NotServedLine className="mt-3">这台电脑上的 usuyo 内核还读不了别的工具导出的文件；现在能用上面「平台上存过的」，或者把链接一条条收下。</NotServedLine>
      ) : readers.error ? (
        <ErrorLine error={readers.error} className="mt-3" />
      ) : null}
      {list.length > 0 && reader ? (
        <>
          <RadioGroup aria-label="用哪个导入插件读" value={reader.id} onChange={(id) => { setReaderId(id); send.reset(); }} className="mt-3.5 grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3">
            {list.map((r) => (
              <ReaderCard key={r.id} reader={r} />
            ))}
          </RadioGroup>
          <DropZone
            aria-label="把文件拖到这里"
            onDrop={(e) => {
              const item = e.items.find((i) => i.kind === "file");
              if (item && item.kind === "file") void item.getFile().then((f) => pick([f]));
            }}
            className="mt-4 flex flex-wrap items-center gap-3 rounded-card border border-dashed border-border2 px-4 py-4 outline-none data-[drop-target]:border-label data-[drop-target]:bg-chip/60 data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
          >
            <FileTrigger acceptedFileTypes={acceptOf(reader.patterns)} onSelect={pick}>
              <Button variant="outline" size="sm">
                <FileUp size={14} aria-hidden="true" />
                选一个文件
              </Button>
            </FileTrigger>
            {file ? (
              <span className="min-w-0 flex-1">
                <span className="mono block truncate text-tag text-text">{file.name}</span>
                <span className="block text-small text-sub">{sizeText(file.size)}</span>
              </span>
            ) : (
              <Text slot="label" className="min-w-0 flex-1 text-note text-sub">
                或者把文件拖到这里。{reader.name}读 <span className="mono text-tag text-label">{reader.patterns.join("  ")}</span>
              </Text>
            )}
            {file ? (
              <Button variant="soft" size="sm" isDisabled={send.isPending} onPress={() => send.mutate({ reader: reader.id, file }, { onSuccess: (v) => onStarted(v.id) })}>
                {send.isPending ? "在交给插件……" : `用${reader.name}读一遍`}
              </Button>
            ) : null}
          </DropZone>
          {send.error ? <ErrorLine error={send.error} className="mt-2" /> : null}
        </>
      ) : null}
      <p className="m-0 mt-3 text-small text-sub">
        别的格式，可以去
        <Link to="/plugins" className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text">
          获取插件
        </Link>
        找读它的导入插件。
      </p>
    </section>
  );
}

function ReaderCard({ reader }: { reader: FileReader }) {
  return (
    <Radio
      value={reader.id}
      className="press cursor-pointer rounded-card border border-border bg-card p-4 outline-none data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:border-border2 data-[selected]:border-label"
    >
      <div className="text-body font-medium text-text">{reader.name}</div>
      <div className="text-small text-sub">{reader.plugin}</div>
      <div className="mt-1.5 text-small text-label">{reader.reads}</div>
      <div className="mt-1 text-small text-sub">读得出：{reader.lists.join("、")}</div>
      {reader.note ? <div className="mt-1 text-small text-faint">{reader.note}</div> : null}
    </Radio>
  );
}
