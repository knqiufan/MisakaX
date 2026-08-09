import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { LoaderCircle } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { artifactsIpc, type ArtifactMetadata, type ArtifactPreview } from "@/lib/ipc";

interface ArtifactPreviewDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  sessionId: string;
  metadata: ArtifactMetadata;
  preview: ArtifactPreview | null;
  onExport: () => void;
}

export function ArtifactPreviewDialog({
  open,
  onOpenChange,
  sessionId,
  metadata,
  preview,
  onExport,
}: ArtifactPreviewDialogProps) {
  const { t } = useTranslation("chat");
  const [base64, setBase64] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open || !preview || preview.kind === "text" || preview.kind === "csv" || preview.kind === "download_only") {
      return;
    }
    let active = true;
    setBase64(null);
    setError(null);
    artifactsIpc
      .readPreviewBase64(sessionId, metadata.artifact_id)
      .then((value) => active && setBase64(value))
      .catch(() => active && setError(t("richContent.preview.failed")));
    return () => {
      active = false;
    };
  }, [metadata.artifact_id, open, preview, sessionId, t]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="grid max-h-[85vh] max-w-4xl grid-rows-[auto_minmax(0,1fr)_auto] overflow-clip p-0 sm:max-w-4xl">
        <DialogHeader className="border-b border-border/50 px-5 py-4 pr-12">
          <DialogTitle>{metadata.display_name}</DialogTitle>
          <DialogDescription>
            {metadata.media_type} · {formatBytes(metadata.byte_size)}
          </DialogDescription>
        </DialogHeader>
        <div className="min-h-0 overflow-auto px-5 py-4">
          {preview?.kind === "text" ? (
            <TextPreview text={preview.text ?? ""} truncated={preview.truncated} />
          ) : preview?.kind === "csv" ? (
            <CsvPreview text={preview.text ?? ""} truncated={preview.truncated} />
          ) : error ? (
            <p className="text-sm text-destructive" role="alert">{error}</p>
          ) : base64 ? (
            <BinaryPreview kind={preview?.kind} base64={base64} mediaType={metadata.media_type} />
          ) : preview?.kind === "download_only" ? (
            <p className="text-sm text-muted-foreground">{t("richContent.preview.downloadOnly")}</p>
          ) : (
            <div className="flex min-h-48 items-center justify-center text-sm text-muted-foreground">
              <LoaderCircle className="mr-2 size-4 animate-spin" aria-hidden />
              {t("richContent.preview.loading")}
            </div>
          )}
        </div>
        <DialogFooter className="border-t border-border/50 px-5 py-3 sm:justify-between">
          <p className="text-xs text-muted-foreground">{t("richContent.preview.readOnly")}</p>
          <Button size="sm" variant="outline" onClick={onExport}>{t("richContent.download")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function BinaryPreview({ kind, base64, mediaType }: { kind: ArtifactPreview["kind"] | undefined; base64: string; mediaType: string }) {
  if (kind === "image") return <ImagePreview base64={base64} mediaType={mediaType} />;
  if (kind === "pdf") return <PdfPreview base64={base64} />;
  if (kind === "spreadsheet") return <SpreadsheetPreview base64={base64} />;
  if (kind === "document") return <DocumentPreview base64={base64} />;
  return null;
}

function TextPreview({ text, truncated }: { text: string; truncated: boolean }) {
  const { t } = useTranslation("chat");
  return (
    <div className="space-y-2">
      {truncated ? <p className="text-xs text-muted-foreground">{t("richContent.preview.truncated")}</p> : null}
      <pre className="max-h-[60vh] overflow-auto whitespace-pre-wrap break-words rounded-lg border border-border/45 bg-muted/30 p-3 font-mono text-xs leading-5 text-foreground">{text}</pre>
    </div>
  );
}

function CsvPreview({ text, truncated }: { text: string; truncated: boolean }) {
  const { t } = useTranslation("chat");
  const rows = parseCsv(text).slice(0, 200).map((row) => row.slice(0, 50));
  return (
    <div className="space-y-2">
      {truncated ? <p className="text-xs text-muted-foreground">{t("richContent.preview.truncated")}</p> : null}
      <div className="overflow-auto rounded-lg border border-border/45">
        <table className="w-full min-w-[420px] text-left text-xs">
          <tbody>
            {rows.map((row, rowIndex) => (
              <tr key={rowIndex} className="border-b border-border/30 last:border-0">
                {row.map((cell, cellIndex) => (
                  <td key={cellIndex} className="max-w-64 truncate px-2 py-1.5">{cell}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

function ImagePreview({ base64, mediaType }: { base64: string; mediaType: string }) {
  const [zoom, setZoom] = useState(1);
  const { t } = useTranslation("chat");
  const source = `data:${mediaType};base64,${base64}`;
  return (
    <div className="space-y-3">
      <div className="max-h-[58vh] overflow-auto rounded-lg border border-border/45 bg-muted/20 p-3 text-center">
        <img src={source} alt={t("richContent.image.previewAlt")} style={{ transform: `scale(${zoom})`, transformOrigin: "center" }} className="max-h-[52vh] max-w-full object-contain" />
      </div>
      <div className="flex items-center gap-2">
        <Button size="xs" variant="outline" onClick={() => setZoom((value) => Math.max(0.5, value - 0.25))}>{t("richContent.image.zoomOut")}</Button>
        <Button size="xs" variant="outline" onClick={() => setZoom(1)}>{t("richContent.image.fit")}</Button>
        <Button size="xs" variant="outline" onClick={() => setZoom((value) => Math.min(3, value + 0.25))}>{t("richContent.image.zoomIn")}</Button>
      </div>
    </div>
  );
}

function PdfPreview({ base64 }: { base64: string }) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [error, setError] = useState(false);
  const [pageNumber, setPageNumber] = useState(1);
  const [pageCount, setPageCount] = useState<number | null>(null);
  useEffect(() => {
    let cancelled = false;
    const task = (async () => {
      try {
        const pdfjs = await import("pdfjs-dist");
        pdfjs.GlobalWorkerOptions.workerSrc = new URL("pdfjs-dist/build/pdf.worker.min.mjs", import.meta.url).toString();
        const document = await pdfjs.getDocument({ data: base64ToBytes(base64) }).promise;
        if (document.numPages > 200) throw new Error("PDF page limit exceeded");
        if (!cancelled) setPageCount(document.numPages);
        const page = await document.getPage(Math.min(pageNumber, document.numPages));
        const viewport = page.getViewport({ scale: 1.25 });
        const canvas = canvasRef.current;
        if (!canvas || cancelled) return;
        canvas.width = Math.ceil(viewport.width);
        canvas.height = Math.ceil(viewport.height);
        const context = canvas.getContext("2d");
        if (!context) throw new Error("2D canvas unavailable");
        await page.render({ canvas, canvasContext: context, viewport }).promise;
      } catch {
        if (!cancelled) setError(true);
      }
    })();
    void task;
    return () => { cancelled = true; };
  }, [base64, pageNumber]);
  const { t } = useTranslation("chat");
  if (error) return <p className="text-sm text-muted-foreground">{t("richContent.preview.failed")}</p>;
  return (
    <div className="space-y-3">
      <div className="overflow-auto rounded-lg border border-border/45 bg-muted/20 p-3"><canvas ref={canvasRef} className="mx-auto max-w-full" aria-label={t("richContent.preview.pdfPage")} /></div>
      {pageCount && pageCount > 1 ? (
        <div className="flex items-center justify-end gap-2">
          <Button size="xs" variant="outline" disabled={pageNumber <= 1} onClick={() => setPageNumber((value) => value - 1)}>{t("richContent.preview.previousPage")}</Button>
          <span className="text-xs text-muted-foreground">{t("richContent.preview.page", { current: pageNumber, total: pageCount })}</span>
          <Button size="xs" variant="outline" disabled={pageNumber >= pageCount} onClick={() => setPageNumber((value) => value + 1)}>{t("richContent.preview.nextPage")}</Button>
        </div>
      ) : null}
    </div>
  );
}

function SpreadsheetPreview({ base64 }: { base64: string }) {
  const [sheets, setSheets] = useState<Array<{ name: string; rows: unknown[][] }> | null>(null);
  const [activeSheet, setActiveSheet] = useState(0);
  const [error, setError] = useState(false);
  useEffect(() => {
    let active = true;
    void import("xlsx")
      .then((xlsx) => {
        const workbook = xlsx.read(base64ToBytes(base64), { type: "array", dense: false });
        const nextSheets = workbook.SheetNames.slice(0, 12).flatMap((name) => {
          const sheet = workbook.Sheets[name];
          if (!sheet) return [];
          const rows = xlsx.utils.sheet_to_json<unknown[]>(sheet, { header: 1, raw: true, defval: "" })
            .slice(0, 200)
            .map((row) => row.slice(0, 50));
          return [{ name, rows }];
        });
        if (active) {
          setActiveSheet(0);
          setSheets(nextSheets);
        }
      })
      .catch(() => active && setError(true));
    return () => { active = false; };
  }, [base64]);
  const { t } = useTranslation("chat");
  if (error) return <p className="text-sm text-muted-foreground">{t("richContent.preview.failed")}</p>;
  if (!sheets) return <PreviewLoader />;
  const sheet = sheets[activeSheet];
  if (!sheet) return <p className="text-sm text-muted-foreground">{t("richContent.preview.failed")}</p>;
  return (
    <div className="space-y-3">
      <div className="flex gap-1 overflow-x-auto border-b border-border/40 pb-2">
        {sheets.map((item, index) => (
          <Button key={item.name} size="xs" variant={index === activeSheet ? "secondary" : "ghost"} onClick={() => setActiveSheet(index)}>{item.name}</Button>
        ))}
      </div>
      <div className="overflow-x-auto rounded-lg border border-border/45"><table className="w-full text-left text-xs"><tbody>{sheet.rows.map((row, rowIndex) => <tr key={rowIndex} className="border-b border-border/30 last:border-0">{row.map((cell, cellIndex) => <td key={cellIndex} className="max-w-64 truncate px-2 py-1.5">{String(cell)}</td>)}</tr>)}</tbody></table></div>
    </div>
  );
}

function DocumentPreview({ base64 }: { base64: string }) {
  const [text, setText] = useState<string | null>(null);
  const [error, setError] = useState(false);
  useEffect(() => {
    let active = true;
    void import("mammoth/mammoth.browser")
      .then(async (mammoth) => {
        const result = await mammoth.convertToHtml({ arrayBuffer: base64ToArrayBuffer(base64) });
        const parsed = new DOMParser().parseFromString(result.value, "text/html");
        if (active) setText((parsed.body.textContent ?? "").slice(0, 512_000));
      })
      .catch(() => active && setError(true));
    return () => { active = false; };
  }, [base64]);
  const { t } = useTranslation("chat");
  if (error) return <p className="text-sm text-muted-foreground">{t("richContent.preview.failed")}</p>;
  if (text === null) return <PreviewLoader />;
  return <article className="whitespace-pre-wrap break-words rounded-lg border border-border/45 bg-muted/20 p-4 text-sm leading-7">{text}</article>;
}

function PreviewLoader() {
  const { t } = useTranslation("chat");
  return <div className="flex min-h-40 items-center justify-center text-sm text-muted-foreground"><LoaderCircle className="mr-2 size-4 animate-spin" aria-hidden />{t("richContent.preview.loading")}</div>;
}

function base64ToBytes(value: string): Uint8Array {
  const binary = window.atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
  return bytes;
}

function base64ToArrayBuffer(value: string): ArrayBuffer {
  const bytes = base64ToBytes(value);
  return new Uint8Array(bytes).buffer;
}

function parseCsv(text: string): string[][] {
  const rows: string[][] = [[]];
  let value = "";
  let quoted = false;
  for (let index = 0; index < text.length; index += 1) {
    const character = text[index];
    if (character === '"' && text[index + 1] === '"' && quoted) {
      value += '"';
      index += 1;
    } else if (character === '"') {
      quoted = !quoted;
    } else if (character === "," && !quoted) {
      rows[rows.length - 1].push(value);
      value = "";
    } else if ((character === "\n" || character === "\r") && !quoted) {
      if (character === "\r" && text[index + 1] === "\n") index += 1;
      rows[rows.length - 1].push(value);
      rows.push([]);
      value = "";
    } else {
      value += character;
    }
  }
  rows[rows.length - 1].push(value);
  return rows.filter((row) => row.length > 1 || row[0] !== "");
}

function formatBytes(value: number): string {
  return new Intl.NumberFormat(undefined, { style: "unit", unit: "byte", unitDisplay: "narrow", notation: "compact" }).format(value);
}
