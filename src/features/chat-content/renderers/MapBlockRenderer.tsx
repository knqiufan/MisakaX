import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Copy, Download, List, LoaderCircle, MapPinned, RotateCcw } from "lucide-react";
import { useTranslation } from "react-i18next";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Button } from "@/components/ui/button";
import { artifactsIpc } from "@/lib/ipc";
import type { Map as MapLibreMap, StyleSpecification } from "maplibre-gl";
import type { BlockRendererProps } from "../renderer-registry";
import { RichContentCard } from "../RichContentCard";
import { readMapSpec, type MapSpecV1 } from "./map-types";
import { NoticeBlockRenderer } from "./NoticeBlockRenderer";

const LOCAL_MAP_STYLE: StyleSpecification = {
  version: 8,
  sources: {},
  layers: [{ id: "background", type: "background", paint: { "background-color": "#18181b" } }],
};

export function MapBlockRenderer({ block, sessionId }: BlockRendererProps) {
  const { t } = useTranslation("chat");
  const spec = useMemo(() => readMapSpec(block), [block]);
  const [showFeatures, setShowFeatures] = useState(false);
  const [resetNonce, setResetNonce] = useState(0);
  const [exporting, setExporting] = useState(false);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const coordinates = useMemo(() => JSON.stringify(spec?.feature_collection ?? {}, null, 2), [spec]);

  const copyCoordinates = useCallback(() => {
    void writeText(coordinates).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1600);
    }).catch(() => setError(t("richContent.map.copyFailed")));
  }, [coordinates, t]);

  const exportGeojson = useCallback(async () => {
    if (!spec) return;
    setExporting(true);
    setError(null);
    let artifactId: string | null = null;
    try {
      const metadata = await artifactsIpc.exportMapGeojson(sessionId, block.message_id, spec);
      artifactId = metadata.artifact_id;
      await artifactsIpc.export(sessionId, metadata.artifact_id);
    } catch {
      setError(t("richContent.map.exportFailed"));
    } finally {
      if (artifactId) void artifactsIpc.expire(sessionId, artifactId);
      setExporting(false);
    }
  }, [block.message_id, sessionId, spec, t]);

  if (!spec) return <NoticeBlockRenderer block={block} sessionId={sessionId} isStreaming={false} />;
  return (
    <RichContentCard
      title={spec.title}
      meta={spec.attribution || t("richContent.map.localOnly")}
      icon={<MapPinned className="size-4" />}
      status={block.status}
      actions={
        <>
          <Button size="icon-xs" variant="ghost" aria-label={t("richContent.map.copyCoordinates")} onClick={copyCoordinates}>
            <Copy className="size-3.5" />
          </Button>
          <Button size="icon-xs" variant="ghost" aria-label={t("richContent.map.toggleFeatures")} aria-pressed={showFeatures} onClick={() => setShowFeatures((value) => !value)}>
            <List className="size-3.5" />
          </Button>
          {spec.initial_view || spec.bounds || spec.markers.length > 0 ? (
            <Button size="icon-xs" variant="ghost" aria-label={t("richContent.map.resetView")} onClick={() => setResetNonce((value) => value + 1)}>
              <RotateCcw className="size-3.5" />
            </Button>
          ) : null}
          <Button size="icon-xs" variant="ghost" aria-label={t("richContent.map.exportGeojson")} disabled={exporting} onClick={exportGeojson}>
            <Download className="size-3.5" />
          </Button>
        </>
      }
      footer={error ? <span className="text-destructive" role="alert">{error}</span> : copied ? t("richContent.map.copied") : null}
    >
      <LocalMap key={resetNonce} spec={spec} />
      {showFeatures ? <FeatureList spec={spec} /> : null}
    </RichContentCard>
  );
}

function LocalMap({ spec }: { spec: MapSpecV1 }) {
  const { t } = useTranslation("chat");
  const elementRef = useRef<HTMLDivElement>(null);
  const [unavailable, setUnavailable] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;
    setUnavailable(false);
    setLoading(true);
    let disposed = false;
    let map: MapLibreMap | null = null;
    let resizeObserver: ResizeObserver | null = null;
    void (async () => {
      try {
        await import("maplibre-gl/dist/maplibre-gl.css");
        const maplibre = await import("maplibre-gl");
        if (disposed) return;
        const firstMarker = spec.markers?.[0];
        const initialView = spec.initial_view;
        const richMap = new maplibre.Map({
          container: element,
          style: LOCAL_MAP_STYLE,
          center: initialView
            ? [initialView.longitude, initialView.latitude]
            : firstMarker ? [firstMarker.longitude, firstMarker.latitude] : [0, 0],
          zoom: initialView?.zoom ?? (firstMarker ? 5 : 1),
          attributionControl: false,
        });
        map = richMap;
        resizeObserver = new ResizeObserver(() => richMap.resize());
        resizeObserver.observe(element);
        richMap.on("load", () => {
          if (disposed) return;
          setLoading(false);
          richMap.addSource("rich-content-data", { type: "geojson", data: spec.feature_collection as never });
          richMap.addLayer({ id: "rich-content-fill", type: "fill", source: "rich-content-data", paint: { "fill-color": "#a1a1aa", "fill-opacity": 0.16 } });
          richMap.addLayer({ id: "rich-content-line", type: "line", source: "rich-content-data", paint: { "line-color": "#d4d4d8", "line-width": 2 } });
          richMap.addLayer({ id: "rich-content-point", type: "circle", source: "rich-content-data", paint: { "circle-color": "#d4d4d8", "circle-radius": 5 } });
          spec.markers?.forEach((marker) => {
            new maplibre.Marker({ color: "#a1a1aa" })
              .setLngLat([marker.longitude, marker.latitude])
              .addTo(richMap);
          });
          if (spec.bounds) {
            richMap.fitBounds(
              [[spec.bounds.west, spec.bounds.south], [spec.bounds.east, spec.bounds.north]],
              { padding: 32, maxZoom: 16, duration: 0 },
            );
          }
        });
        richMap.on("error", () => {
          if (!disposed) {
            setLoading(false);
            setUnavailable(true);
          }
        });
      } catch {
        if (!disposed) {
          setLoading(false);
          setUnavailable(true);
        }
      }
    })();
    return () => {
      disposed = true;
      resizeObserver?.disconnect();
      map?.remove();
    };
  }, [spec]);

  if (unavailable) return <FeatureList spec={spec} label={t("richContent.map.fallbackFeatures")} />;
  return (
    <div className="relative h-[280px] min-w-0 bg-muted/20" aria-busy={loading}>
      <div ref={elementRef} className="h-full w-full" role="img" aria-label={t("richContent.map.visualization")} />
      {loading ? <LoaderCircle className="absolute inset-0 m-auto size-4 animate-spin text-muted-foreground" aria-label={t("richContent.preview.loading")} /> : null}
    </div>
  );
}

function FeatureList({ spec, label }: { spec: MapSpecV1; label?: string }) {
  const { t } = useTranslation("chat");
  return (
    <div className="border-t border-border/35 px-3 py-3 text-xs">
      {label ? <p className="mb-2 text-muted-foreground">{label}</p> : null}
      <p className="text-muted-foreground">{t("richContent.map.featureCount", { count: spec.feature_collection.features.length })}</p>
      <ul className="mt-2 space-y-1.5">
        {spec.feature_collection.features.slice(0, 200).map((feature, index) => (
          <li key={`${feature.geometry.type}-${index}`} className="rounded-md bg-muted/40 px-2.5 py-2 text-foreground">
            <span className="font-medium">{feature.geometry.type}</span>
            {feature.properties && Object.keys(feature.properties).length > 0 ? <span className="ml-2 text-muted-foreground">{Object.entries(feature.properties).map(([key, value]) => `${key}: ${value}`).join(" · ")}</span> : null}
          </li>
        ))}
      </ul>
      {spec.feature_collection.features.length > 200 ? <p className="mt-2 text-muted-foreground">{t("richContent.map.featureListTruncated", { count: spec.feature_collection.features.length })}</p> : null}
    </div>
  );
}
