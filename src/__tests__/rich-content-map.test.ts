import { describe, expect, it } from "vitest";
import type { ContentBlock } from "@/lib/ipc";
import { readMapSpec } from "@/features/chat-content/renderers/map-types";

function mapBlock(spec: unknown): ContentBlock {
  return {
    id: "block-1",
    message_id: "message-1",
    position: 0,
    schema_version: 1,
    kind: "map",
    status: "ready",
    payload: { spec },
    fallback: { title: "Map", message_key: "richContent.blockUnavailable" },
    generation: 1,
    revision: 1,
    created_at: "2026-08-09T00:00:00Z",
    updated_at: "2026-08-09T00:00:00Z",
  };
}

const validSpec = {
  title: "Hangzhou places",
  feature_collection: {
    type: "FeatureCollection",
    features: [{
      type: "Feature",
      geometry: { type: "Point", coordinates: [120.15, 30.28] },
      properties: { name: "West Lake" },
    }],
  },
  markers: [{ longitude: 120.15, latitude: 30.28, label: "West Lake" }],
};

describe("local rich-content map parser", () => {
  it("accepts bounded local GeoJSON without a remote resource", () => {
    expect(readMapSpec(mapBlock(validSpec))).toMatchObject({
      title: "Hangzhou places",
      feature_collection: { type: "FeatureCollection" },
      markers: [{ label: "West Lake" }],
    });
  });

  it("rejects remote tile configuration and invalid coordinates", () => {
    expect(readMapSpec(mapBlock({ ...validSpec, tile_source_id: "https://tiles.example.com/{z}/{x}/{y}" }))).toBeNull();
    expect(readMapSpec(mapBlock({
      ...validSpec,
      feature_collection: {
        ...validSpec.feature_collection,
        features: [{
          ...validSpec.feature_collection.features[0],
          geometry: { type: "Point", coordinates: [120.15, 91] },
        }],
      },
    }))).toBeNull();
  });
});
