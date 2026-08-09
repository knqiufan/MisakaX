import { useEffect, useState } from "react";
import { Image as ImageIcon, LoaderCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { artifactsIpc, type ArtifactMetadata } from "@/lib/ipc";
import type { BlockRendererProps } from "../renderer-registry";
import { RichContentCard } from "../RichContentCard";
import { readArtifactPayload } from "../types";
import { NoticeBlockRenderer } from "./NoticeBlockRenderer";
import { ArtifactPreviewDialog } from "./ArtifactPreviewDialog";

export function ImageBlockRenderer({ block, sessionId }: BlockRendererProps) {
  const { t } = useTranslation("chat");
  const payload = readArtifactPayload(block);
  const [metadata, setMetadata] = useState<ArtifactMetadata | null>(null);
  const [base64, setBase64] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [open, setOpen] = useState(false);
  useEffect(() => {
    if (!payload) return;
    let active = true;
    void Promise.all([
      artifactsIpc.getMetadata(sessionId, payload.artifact_id),
      artifactsIpc.readPreviewBase64(sessionId, payload.artifact_id),
    ]).then(([nextMetadata, nextBase64]) => {
      if (!active) return;
      setMetadata(nextMetadata);
      setBase64(nextBase64);
    }).catch(() => active && setError(t("richContent.artifact.unavailable")));
    return () => { active = false; };
  }, [payload?.artifact_id, sessionId, t]);
  if (!payload) return <NoticeBlockRenderer block={block} sessionId={sessionId} isStreaming={false} />;
  return (
    <>
      <RichContentCard title={metadata?.display_name ?? payload.display_name ?? t("richContent.image.untitled")} icon={<ImageIcon className="size-4" />} status={block.status} footer={error ? <span className="text-destructive" role="alert">{error}</span> : metadata ? `${metadata.media_type} · ${formatBytes(metadata.byte_size)}${payload.width && payload.height ? ` · ${payload.width} × ${payload.height}` : ""}` : t("richContent.artifact.loading")}>
        <div className="flex min-h-40 items-center justify-center bg-muted/15 p-3">
          {base64 && metadata ? <button type="button" className="cursor-pointer rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-ring" onClick={() => setOpen(true)} aria-label={t("richContent.image.open")}><img src={`data:${metadata.media_type};base64,${base64}`} alt={payload.alt ?? t("richContent.image.previewAlt")} className="max-h-96 max-w-full object-contain" /></button> : error ? <p className="text-sm text-destructive" role="alert">{error}</p> : <LoaderCircle className="size-4 animate-spin text-muted-foreground" aria-label={t("richContent.preview.loading")} />}
        </div>
      </RichContentCard>
      {metadata ? <ArtifactPreviewDialog open={open} onOpenChange={setOpen} sessionId={sessionId} metadata={metadata} preview={{ kind: "image", state: "ready", message_key: null, text: null, truncated: false }} onExport={() => void artifactsIpc.export(sessionId, metadata.artifact_id)} /> : null}
    </>
  );
}

function formatBytes(value: number): string {
  return new Intl.NumberFormat(undefined, { style: "unit", unit: "byte", unitDisplay: "narrow", notation: "compact" }).format(value);
}
