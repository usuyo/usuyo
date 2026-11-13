import { Link } from "@tanstack/react-router";
import { FileUp } from "lucide-react";
import { useRef, useState } from "react";
import { DropZone, FileTrigger, Radio, RadioGroup, Text } from "react-aria-components";
import type { FileReader } from "../api/endpoints";
import { useFileReaders, useImportFile } from "../api/hooks";
import { ErrorLine } from "../components/ErrorLine";
import { sizeText } from "../item/faces";
import { Button } from "../ui/Button";
import { acceptOf } from "./words";

/** 停下的那一次：不是出错，不报。 */
const isStopped = (error: Error | null) => error?.name === "AbortError";

/**
 * 别的工具导出的文件：挑一种读它的方法，选文件（或拖进来）就交过去，交的时候说交了多少、能停下。
 * 内核存一份副本，原文件不动；读出来的和别的导入一样先给你看，你点了才收。
 */
export function FileImport({ onStarted, initial }: { onStarted: (id: string) => void; initial?: string }) {
  const readers = useFileReaders();
  const send = useImportFile();
  const [readerId, setReaderId] = useState<string | undefined>(initial);
  const [file, setFile] = useState<File | null>(null);
  const [sent, setSent] = useState(0);
  const stopper = useRef<AbortController | null>(null);
  const list = readers.data ?? [];
  const reader = list.find((r) => r.id === readerId) ?? list[0];

  const hand = (f: File | undefined) => {
    if (!f || !reader || send.isPending) return;
    setFile(f);
    setSent(0);
    const stop = new AbortController();
    stopper.current = stop;
    send.mutate({ reader: reader.id, file: f, onSent: setSent, signal: stop.signal }, { onSuccess: (v) => onStarted(v.id) });
  };

  return (
    <section aria-labelledby="file-import-h">
      <h2 id="file-import-h" className="m-0 text-section font-semibold text-text">
        别的工具导出的文件
      </h2>
      <p className="m-0 mt-1 text-small text-sub">平台官方给的归档、书签工具和阅读器导出的清单。存的是一份副本，原文件不动；先读出来给你看，你点了才收。</p>
      {readers.error ? <ErrorLine error={readers.error} className="mt-3" /> : null}
      {readers.data && list.length === 0 ? <p className="m-0 mt-3 text-note text-sub">装着的插件里还没有能读文件的。</p> : null}
      {reader ? (
        <>
          <RadioGroup
            aria-label="用哪种方法读"
            value={reader.id}
            isDisabled={send.isPending}
            onChange={(id) => {
              setReaderId(id);
              send.reset();
              setFile(null);
            }}
            className="mt-3.5 grid grid-cols-[repeat(auto-fill,minmax(240px,1fr))] gap-3"
          >
            {list.map((r) => (
              <ReaderCard key={r.id} reader={r} />
            ))}
          </RadioGroup>
          <DropZone
            aria-label="把文件拖到这里"
            isDisabled={send.isPending}
            onDrop={(e) => {
              const item = e.items.find((i) => i.kind === "file");
              if (item && item.kind === "file") void item.getFile().then(hand);
            }}
            className="mt-4 rounded-card border border-dashed border-border2 px-4 py-4 outline-none data-[drop-target]:border-label data-[drop-target]:bg-chip/60 data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent"
          >
            {send.isPending && file ? (
              <Sending file={file} sent={sent} reader={reader} onStop={() => stopper.current?.abort()} />
            ) : (
              <div className="flex flex-wrap items-center gap-3">
                <FileTrigger acceptedFileTypes={acceptOf(reader.patterns)} onSelect={(files) => hand(files?.[0])}>
                  <Button variant="outline" size="sm">
                    <FileUp size={14} aria-hidden="true" />
                    选一个文件
                  </Button>
                </FileTrigger>
                <Text slot="label" className="min-w-0 flex-1 text-note text-sub">
                  或者把文件拖到这里，选好就交给{reader.plugin}的{reader.name}读。
                </Text>
              </div>
            )}
          </DropZone>
          {file && !send.isPending && (send.error || send.isSuccess) ? (
            <p className="m-0 mt-2 text-small text-sub">
              <span className="mono text-tag text-faint">{file.name}</span>
              {isStopped(send.error) ? " · 停下了，交了一半的没有留下" : ""}
            </p>
          ) : null}
          {send.error && !isStopped(send.error) ? <ErrorLine error={send.error} className="mt-1" /> : null}
        </>
      ) : null}
      <p className="m-0 mt-3 text-small text-sub">
        别的格式，可以去
        <Link to="/plugins" className="mx-0.5 text-label underline decoration-border2 underline-offset-4 hover:text-text">
          获取插件
        </Link>
        找读它的插件。
      </p>
    </section>
  );
}

/** 交着的时候：交了多少字节、一共多少，一条跟着走的细线；交完了在等插件读第一页。 */
function Sending({ file, sent, reader, onStop }: { file: File; sent: number; reader: FileReader; onStop: () => void }) {
  const whole = sent >= file.size;
  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
      <div className="min-w-0 flex-[1_1_260px]">
        <div className="mono truncate text-tag text-text">{file.name}</div>
        <div role="status" className="mt-0.5 text-small text-sub">
          {whole ? `交完了 ${sizeText(file.size)}，在等${reader.name}读第一页……` : `交了 ${sizeText(sent)}，一共 ${sizeText(file.size)}`}
        </div>
        <div aria-hidden="true" className="mt-2 h-0.5 overflow-hidden rounded-full bg-chip">
          <div
            className="h-full bg-label transition-[width] duration-700 ease-linear motion-reduce:transition-none"
            style={{ width: `${file.size > 0 ? Math.min(1, sent / file.size) * 100 : 0}%` }}
          />
        </div>
      </div>
      <Button variant="outline" size="sm" onPress={onStop}>
        停下
      </Button>
    </div>
  );
}

function ReaderCard({ reader }: { reader: FileReader }) {
  return (
    <Radio
      value={reader.id}
      className="press cursor-pointer rounded-card border border-border bg-card p-4 outline-none data-[disabled]:cursor-default data-[focus-visible]:outline-2 data-[focus-visible]:outline-accent data-[hovered]:border-border2 data-[selected]:border-label"
    >
      <div className="text-body font-medium text-text">{reader.name}</div>
      <div className="text-small text-sub">{reader.plugin}</div>
      <div className="mt-1.5 text-small text-label">{reader.reads}</div>
      {reader.lists.length > 0 ? <div className="mt-1 text-small text-sub">读得出：{reader.lists.join("、")}</div> : null}
      {reader.patterns.length > 0 ? <div className="mono mt-1 text-tag text-faint">{reader.patterns.join("  ")}</div> : null}
      {reader.note ? <div className="mt-1 text-small text-faint">记下的：{reader.note}</div> : null}
    </Radio>
  );
}
