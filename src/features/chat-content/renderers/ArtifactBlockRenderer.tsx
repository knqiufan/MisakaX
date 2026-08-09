import { useCallback, useEffect, useState } from "react";
import { Download, FileText, Eye, LoaderCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { artifactsIpc, type ArtifactMetadata, type ArtifactPreview } from "@/lib/ipc";
import type { BlockRendererProps } from "../renderer-registry";
import { RichContentCard } from "../RichContentCard";
import { readArtifactPayload } from "../types";
import { NoticeBlockRenderer } from "./NoticeBlockRenderer";
import { ArtifactPreviewDialog } from "./ArtifactPreviewDialog";

export function ArtifactBlockRenderer({ block, sessionId }: BlockRendererProps) {
  const { t } = useTranslation("chat");
  const payload = readArtifactPayload(block);
  const [metadata, setMetadata] = useState<ArtifactMetadata | null>(null);
  const [preview, setPreview] = useState<ArtifactPreview | null>(null);
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!payload) return;
    let active = true;
    artifactsIpc.getMetadata(sessionId, payload.artifact_id).then((result) => active && setMetadata(result)).catch(() => active && setError(t("richContent.artifact.unavailable")));
    return () => { active = false; };
  }, [payload?.artifact_id, sessionId, t]);

  const exportArtifact = useCallback(async () => {
    if (!metadata) return;
    try {
      const result = await artifactsIpc.export(sessionId, metadata.artifact_id);
      if (result.status === "saved") setError(null);
    } catch {
      setError(t("richContent.artifact.exportFailed"));
    }
  }, [metadata, sessionId, t]);

  const openPreview = useCallback(async () => {
    if (!metadata) return;
    setLoading(true);
    setError(null);
    try {
      const result = await artifactsIpc.getPreview(sessionId, metadata.artifact_id);
      setPreview(result);
      setOpen(true);
    } catch {
      setError(t("richContent.preview.failed"));
    } finally {
      setLoading(false);
    }
  }, [metadata, sessionId, t]);

  if (!payload) return <NoticeBlockRenderer block={block} sessionId={sessionId} isStreaming={false} />;
  const title = metadata?.display_name ?? payload.display_name ?? t("richContent.artifact.untitled");
  return (
    <>
      <RichContentCard
        title={title}
        icon={<FileText className="size-4" />}
        status={block.status}
        actions={loading ? <LoaderCircle className="size-4 animate-spin" aria-hidden /> : null}
        footer={error ? <span className="text-destructive" role="alert">{error}</span> : metadata ? `${metadata.media_type} · ${formatBytes(metadata.byte_size)} · ${t(`richContent.origin.${metadata.origin_kind}`)} · ${formatCreatedAt(metadata.created_at)}` : t("richContent.artifact.loading")}
      >
        <div className="flex flex-wrap gap-2 px-3 py-3">
          <Button size="sm" variant="outline" onClick={openPreview} disabled={!metadata || loading}><Eye className="size-3.5" />{t("richContent.preview.open")}</Button>
          <Button size="sm" variant="outline" onClick={exportArtifact} disabled={!metadata}><Download className="size-3.5" />{t("richContent.download")}</Button>
        </div>
      </RichContentCard>
      {metadata ? <ArtifactPreviewDialog open={open} onOpenChange={setOpen} sessionId={sessionId} metadata={metadata} preview={preview} onExport={exportArtifact} /> : null}
    </>
  );
}

function formatBytes(value: number): string {
  return new Intl.NumberFormat(undefined, { style: "unit", unit: "byte", unitDisplay: "narrow", notation: "compact" }).format(value);
}

function formatCreatedAt(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.valueOf()) ? "—" : new Intl.DateTimeFormat(undefined, { dateStyle: "short", timeStyle: "short" }).format(date);
}
