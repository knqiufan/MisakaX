import type { ContentBlock } from "@/lib/ipc";

export interface GeoJsonFeature {
  type: "Feature";
  geometry: {
    type: "Point" | "LineString" | "Polygon";
    coordinates: unknown;
  };
  properties?: Record<string, string>;
}

export interface MapSpecV1 {
  title: string;
  summary?: string;
  feature_collection: {
    type: "FeatureCollection";
    features: GeoJsonFeature[];
  };
  markers: Array<{ longitude: number; latitude: number; label: string }>;
  attribution?: string;
  initial_view?: { longitude: number; latitude: number; zoom: number };
  bounds?: { west: number; south: number; east: number; north: number };
}

const MAX_FEATURES = 10_000;
const MAX_PROPERTIES = 24;
const MAX_LABEL_CHARS = 512;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isString(value: unknown): value is string {
  return typeof value === "string";
}

function isCoordinatePair(value: unknown): value is [number, number] {
  return Array.isArray(value)
    && value.length === 2
    && value.every((coordinate) => typeof coordinate === "number" && Number.isFinite(coordinate))
    && value[0] >= -180
    && value[0] <= 180
    && value[1] >= -90
    && value[1] <= 90;
}

function areCoordinatesValid(value: unknown): boolean {
  if (isCoordinatePair(value)) return true;
  return Array.isArray(value) && value.length > 0 && value.every(areCoordinatesValid);
}

function readView(value: unknown): MapSpecV1["initial_view"] | null {
  if (
    !isRecord(value)
    || typeof value.longitude !== "number"
    || typeof value.latitude !== "number"
    || typeof value.zoom !== "number"
    || !Number.isFinite(value.zoom)
    || value.zoom < 0
    || value.zoom > 22
    || !isCoordinatePair([value.longitude, value.latitude])
  ) {
    return null;
  }
  return { longitude: value.longitude, latitude: value.latitude, zoom: value.zoom };
}

function readBounds(value: unknown): MapSpecV1["bounds"] | null {
  if (
    !isRecord(value)
    || typeof value.west !== "number"
    || typeof value.south !== "number"
    || typeof value.east !== "number"
    || typeof value.north !== "number"
    || !isCoordinatePair([value.west, value.south])
    || !isCoordinatePair([value.east, value.north])
    || value.west > value.east
    || value.south > value.north
  ) {
    return null;
  }
  return { west: value.west, south: value.south, east: value.east, north: value.north };
}

export function readMapSpec(block: ContentBlock): MapSpecV1 | null {
  if (!isRecord(block.payload)) return null;
  const candidate = isRecord(block.payload.spec) ? block.payload.spec : block.payload;
  if (
    !isString(candidate.title)
    || candidate.title.trim().length === 0
    || candidate.title.length > MAX_LABEL_CHARS
    || candidate.tile_source_id !== undefined
    || !isRecord(candidate.feature_collection)
  ) {
    return null;
  }
  const collection = candidate.feature_collection;
  if (
    collection.type !== "FeatureCollection"
    || !Array.isArray(collection.features)
    || collection.features.length > MAX_FEATURES
  ) {
    return null;
  }
  const features = collection.features.flatMap((feature) => {
    if (!isRecord(feature) || feature.type !== "Feature" || !isRecord(feature.geometry)) return [];
    const geometry = feature.geometry;
    if (
      !isString(geometry.type)
      || !["Point", "LineString", "Polygon"].includes(geometry.type)
      || !areCoordinatesValid(geometry.coordinates)
    ) {
      return [];
    }
    if (feature.properties !== undefined && !isRecord(feature.properties)) return [];
    const entries = feature.properties ? Object.entries(feature.properties) : [];
    if (
      entries.length > MAX_PROPERTIES
      || entries.some(([key, value]) => !isString(value) || key.length > MAX_LABEL_CHARS || value.length > MAX_LABEL_CHARS)
    ) {
      return [];
    }
    const properties: Record<string, string> | undefined = entries.length > 0
      ? Object.fromEntries(entries) as Record<string, string>
      : undefined;
    return [{
      type: "Feature" as const,
      geometry: { type: geometry.type as GeoJsonFeature["geometry"]["type"], coordinates: geometry.coordinates },
      properties,
    }];
  });
  if (features.length !== collection.features.length) return null;

  const rawMarkers = candidate.markers ?? [];
  if (!Array.isArray(rawMarkers) || rawMarkers.length > MAX_FEATURES) return null;
  const markers = rawMarkers.flatMap((marker) => {
    if (
      !isRecord(marker)
      || typeof marker.longitude !== "number"
      || typeof marker.latitude !== "number"
      || !isCoordinatePair([marker.longitude, marker.latitude])
      || !isString(marker.label)
      || marker.label.length > MAX_LABEL_CHARS
    ) {
      return [];
    }
    return [{ longitude: marker.longitude, latitude: marker.latitude, label: marker.label }];
  });
  if (markers.length !== rawMarkers.length) return null;

  const initialView = candidate.initial_view === undefined ? undefined : readView(candidate.initial_view);
  const bounds = candidate.bounds === undefined ? undefined : readBounds(candidate.bounds);
  if ((candidate.initial_view !== undefined && !initialView) || (candidate.bounds !== undefined && !bounds)) return null;

  return {
    title: candidate.title,
    summary: isString(candidate.summary) && candidate.summary.length <= MAX_LABEL_CHARS ? candidate.summary : undefined,
    feature_collection: { type: "FeatureCollection", features },
    markers,
    attribution: isString(candidate.attribution) && candidate.attribution.length <= MAX_LABEL_CHARS ? candidate.attribution : undefined,
    initial_view: initialView ?? undefined,
    bounds: bounds ?? undefined,
  };
}
