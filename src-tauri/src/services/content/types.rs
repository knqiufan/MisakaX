use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::services::artifacts::ContentSafetyPolicy;

pub const CONTENT_BLOCK_SCHEMA_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentBlockKind {
    Markdown,
    Chart,
    Map,
    Artifact,
    Image,
    Notice,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockStatus {
    Pending,
    Ready,
    Failed,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BlockFallback {
    pub title: String,
    pub message_key: String,
    #[serde(default)]
    pub params: BTreeMap<String, String>,
    #[serde(default)]
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    pub id: String,
    pub message_id: String,
    pub position: i64,
    pub schema_version: u16,
    pub kind: ContentBlockKind,
    pub status: BlockStatus,
    pub payload: Value,
    #[serde(default)]
    pub fallback: BlockFallback,
    #[serde(default)]
    pub generation: u64,
    #[serde(default)]
    pub revision: u64,
    pub created_at: String,
    pub updated_at: String,
}

impl ContentBlock {
    pub fn new(
        message_id: String,
        position: i64,
        kind: ContentBlockKind,
        status: BlockStatus,
        payload: Value,
        fallback: BlockFallback,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_id,
            position,
            schema_version: CONTENT_BLOCK_SCHEMA_VERSION,
            kind,
            status,
            payload,
            fallback,
            generation: 0,
            revision: 1,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn validate(&self, policy: &ContentSafetyPolicy) -> Result<(), String> {
        if self.schema_version != CONTENT_BLOCK_SCHEMA_VERSION {
            return Err("CONTENT_BLOCK_UNSUPPORTED".to_string());
        }
        if self.message_id.trim().is_empty() || self.position < 0 {
            return Err("CONTENT_BLOCK_INVALID".to_string());
        }
        if serde_json::to_vec(&self.payload)
            .map_err(|_| "CONTENT_BLOCK_INVALID".to_string())?
            .len()
            > policy.max_block_payload_bytes
        {
            return Err("CONTENT_BLOCK_INVALID".to_string());
        }

        match self.kind {
            ContentBlockKind::Markdown => validate_markdown_payload(&self.payload, policy),
            ContentBlockKind::Chart => ChartSpecV1::from_block_payload(&self.payload, policy),
            ContentBlockKind::Map => MapSpecV1::from_block_payload(&self.payload, policy),
            ContentBlockKind::Artifact | ContentBlockKind::Image => {
                let id = self
                    .payload
                    .get("artifact_id")
                    .and_then(Value::as_str)
                    .filter(|value| uuid::Uuid::parse_str(value).is_ok());
                id.ok_or_else(|| "CONTENT_BLOCK_INVALID".to_string())?;
                reject_active_content(&self.payload)
            }
            ContentBlockKind::Notice => {
                let key = self.payload.get("message_key").and_then(Value::as_str);
                if key.is_some_and(|value| !value.trim().is_empty()) {
                    Ok(())
                } else {
                    Err("CONTENT_BLOCK_INVALID".to_string())
                }
            }
            ContentBlockKind::Unknown => Err("CONTENT_BLOCK_UNSUPPORTED".to_string()),
        }
    }
}

fn validate_markdown_payload(payload: &Value, policy: &ContentSafetyPolicy) -> Result<(), String> {
    let text = payload
        .get("text")
        .and_then(Value::as_str)
        .ok_or_else(|| "CONTENT_BLOCK_INVALID".to_string())?;
    if text.len() > policy.max_markdown_chars {
        return Err("CONTENT_BLOCK_INVALID".to_string());
    }
    Ok(())
}

fn reject_active_content(payload: &Value) -> Result<(), String> {
    let encoded = payload.to_string().to_ascii_lowercase();
    if encoded.contains("file:")
        || encoded.contains("javascript:")
        || encoded.contains("<script")
        || encoded.contains("<svg")
        || encoded.contains("http://")
        || encoded.contains("https://")
    {
        return Err("CONTENT_BLOCK_INVALID".to_string());
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    Line,
    Bar,
    Area,
    Scatter,
    Pie,
    Metric,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDatum {
    pub x: String,
    pub y: f64,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    pub name: String,
    pub values: Vec<ChartDatum>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSpecV1 {
    pub chart_type: ChartType,
    pub title: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub x_label: Option<String>,
    #[serde(default)]
    pub y_label: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub series: Vec<ChartSeries>,
}

impl ChartSpecV1 {
    pub fn validate(&self, policy: &ContentSafetyPolicy) -> Result<(), String> {
        let spec = self;
        if spec.title.trim().is_empty() || spec.title.len() > policy.max_label_chars {
            return Err("CHART_SPEC_INVALID".to_string());
        }
        if spec.series.is_empty() || spec.series.len() > policy.max_chart_series {
            return Err("CHART_SPEC_INVALID".to_string());
        }
        let mut points = 0usize;
        for series in &spec.series {
            if series.name.trim().is_empty() || series.name.len() > policy.max_label_chars {
                return Err("CHART_SPEC_INVALID".to_string());
            }
            points += series.values.len();
            if series.values.iter().any(|item| {
                !item.y.is_finite()
                    || item.x.len() > policy.max_label_chars
                    || item
                        .label
                        .as_ref()
                        .is_some_and(|label| label.len() > policy.max_label_chars)
            }) {
                return Err("CHART_SPEC_INVALID".to_string());
            }
        }
        (points <= policy.max_chart_points)
            .then_some(())
            .ok_or_else(|| "CHART_SPEC_INVALID".to_string())
    }

    fn from_block_payload(payload: &Value, policy: &ContentSafetyPolicy) -> Result<(), String> {
        reject_active_content(payload).map_err(|_| "CHART_SPEC_INVALID".to_string())?;
        let spec: Self = serde_json::from_value(
            payload
                .get("spec")
                .cloned()
                .unwrap_or_else(|| payload.clone()),
        )
        .map_err(|_| "CHART_SPEC_INVALID".to_string())?;
        spec.validate(policy)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonFeatureCollection {
    #[serde(rename = "type")]
    pub type_name: String,
    pub features: Vec<GeoJsonFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonFeature {
    #[serde(rename = "type")]
    pub type_name: String,
    pub geometry: GeoJsonGeometry,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoJsonGeometry {
    #[serde(rename = "type")]
    pub type_name: String,
    pub coordinates: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapMarker {
    pub longitude: f64,
    pub latitude: f64,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapInitialView {
    pub longitude: f64,
    pub latitude: f64,
    pub zoom: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapBounds {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapSpecV1 {
    pub title: String,
    #[serde(default)]
    pub summary: String,
    pub feature_collection: GeoJsonFeatureCollection,
    #[serde(default)]
    pub markers: Vec<MapMarker>,
    #[serde(default)]
    pub attribution: String,
    #[serde(default)]
    pub initial_view: Option<MapInitialView>,
    #[serde(default)]
    pub bounds: Option<MapBounds>,
    #[serde(default)]
    pub tile_source_id: Option<String>,
}

impl MapSpecV1 {
    pub fn validate(&self, policy: &ContentSafetyPolicy) -> Result<(), String> {
        let spec = self;
        if spec.title.trim().is_empty()
            || spec.feature_collection.type_name != "FeatureCollection"
            || spec.feature_collection.features.len() > policy.max_map_features
            || spec.markers.len() > policy.max_map_features
            || spec.tile_source_id.is_some()
        {
            return Err("MAP_SPEC_INVALID".to_string());
        }
        for marker in &spec.markers {
            if !valid_coordinate(marker.longitude, marker.latitude)
                || marker.label.len() > policy.max_label_chars
            {
                return Err("MAP_SPEC_INVALID".to_string());
            }
        }
        if spec.initial_view.as_ref().is_some_and(|view| {
            !valid_coordinate(view.longitude, view.latitude)
                || !view.zoom.is_finite()
                || !(0.0..=22.0).contains(&view.zoom)
        }) {
            return Err("MAP_SPEC_INVALID".to_string());
        }
        if spec.bounds.as_ref().is_some_and(|bounds| {
            !valid_coordinate(bounds.west, bounds.south)
                || !valid_coordinate(bounds.east, bounds.north)
                || bounds.west > bounds.east
                || bounds.south > bounds.north
        }) {
            return Err("MAP_SPEC_INVALID".to_string());
        }
        for feature in &spec.feature_collection.features {
            if feature.type_name != "Feature"
                || !matches!(
                    feature.geometry.type_name.as_str(),
                    "Point" | "LineString" | "Polygon"
                )
                || feature.properties.len() > policy.max_map_properties
                || feature.properties.iter().any(|(key, value)| {
                    key.len() > policy.max_label_chars || value.len() > policy.max_label_chars
                })
                || !validate_coordinates(&feature.geometry.coordinates)
            {
                return Err("MAP_SPEC_INVALID".to_string());
            }
        }
        Ok(())
    }

    fn from_block_payload(payload: &Value, policy: &ContentSafetyPolicy) -> Result<(), String> {
        reject_active_content(payload).map_err(|_| "MAP_SPEC_INVALID".to_string())?;
        let spec: Self = serde_json::from_value(
            payload
                .get("spec")
                .cloned()
                .unwrap_or_else(|| payload.clone()),
        )
        .map_err(|_| "MAP_SPEC_INVALID".to_string())?;
        spec.validate(policy)
    }
}

fn validate_coordinates(value: &Value) -> bool {
    match value {
        Value::Array(values) if values.len() == 2 && values.iter().all(Value::is_number) => {
            let longitude = values[0].as_f64().unwrap_or(f64::NAN);
            let latitude = values[1].as_f64().unwrap_or(f64::NAN);
            valid_coordinate(longitude, latitude)
        }
        Value::Array(values) if !values.is_empty() => values.iter().all(validate_coordinates),
        _ => false,
    }
}

fn valid_coordinate(longitude: f64, latitude: f64) -> bool {
    longitude.is_finite()
        && latitude.is_finite()
        && (-180.0..=180.0).contains(&longitude)
        && (-90.0..=90.0).contains(&latitude)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_external_map_tile_and_active_payload() {
        let policy = ContentSafetyPolicy::default();
        let block = ContentBlock::new(
            "message".into(),
            0,
            ContentBlockKind::Map,
            BlockStatus::Ready,
            serde_json::json!({
                "spec": {
                    "title": "Map",
                    "feature_collection": {"type": "FeatureCollection", "features": []},
                    "tile_source_id": "https://example.test/{z}/{x}/{y}.png"
                }
            }),
            BlockFallback::default(),
        );
        assert_eq!(block.validate(&policy).unwrap_err(), "MAP_SPEC_INVALID");
    }

    #[test]
    fn validates_safe_chart_spec() {
        let policy = ContentSafetyPolicy::default();
        let block = ContentBlock::new(
            "message".into(),
            0,
            ContentBlockKind::Chart,
            BlockStatus::Ready,
            serde_json::json!({"spec": {
                "chart_type": "line", "title": "Trend",
                "series": [{"name": "Revenue", "values": [{"x": "Jan", "y": 12.0}]}]
            }}),
            BlockFallback::default(),
        );
        assert!(block.validate(&policy).is_ok());
    }

    #[test]
    fn maps_unknown_block_kinds_to_a_non_executing_fallback() {
        let kind = serde_json::from_str::<ContentBlockKind>("\"future_widget\"").unwrap();
        assert_eq!(kind, ContentBlockKind::Unknown);
    }
}
