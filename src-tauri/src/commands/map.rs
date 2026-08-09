use rusqlite::params;
use tauri::State;

use crate::config;
use crate::services::artifacts::{
    ArtifactMetadata, ArtifactOrigin, ArtifactService, ContentSafetyPolicy,
};
use crate::services::content::MapSpecV1;
use crate::AppState;

fn service() -> Result<ArtifactService, String> {
    ArtifactService::new(
        config::artifacts_dir().map_err(|error| error.to_string())?,
        ContentSafetyPolicy::default(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn map_export_geojson(
    state: State<'_, AppState>,
    session_id: String,
    message_id: String,
    spec: MapSpecV1,
) -> Result<ArtifactMetadata, String> {
    if !state.feature_flags.rich_content_write {
        return Err("CONTENT_BLOCK_UNSUPPORTED".to_string());
    }

    let service = service()?;
    spec.validate(service.policy())
        .map_err(|error| error.to_string())?;

    let conn = state.db.lock().map_err(|error| error.to_string())?;
    let message_belongs_to_session = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM messages WHERE id = ?1 AND session_id = ?2)",
            params![message_id, session_id],
            |row| row.get::<_, bool>(0),
        )
        .map_err(|error| error.to_string())?;
    if !message_belongs_to_session {
        return Err("ARTIFACT_ACCESS_DENIED".to_string());
    }

    service
        .register_bytes(
            &conn,
            session_id,
            Some(message_id),
            ArtifactOrigin::Agent,
            "map-data.geojson".to_string(),
            "application/geo+json".to_string(),
            &map_geojson(&spec)?,
        )
        .map_err(|error| error.to_string())
}

fn map_geojson(spec: &MapSpecV1) -> Result<Vec<u8>, String> {
    serde_json::to_vec(&spec.feature_collection).map_err(|_| "MAP_SPEC_INVALID".to_string())
}

#[cfg(test)]
mod tests {
    use super::{map_geojson, MapSpecV1};

    #[test]
    fn geojson_export_contains_only_the_feature_collection() {
        let spec: MapSpecV1 = serde_json::from_value(serde_json::json!({
            "title": "Places",
            "feature_collection": {
                "type": "FeatureCollection",
                "features": [{
                    "type": "Feature",
                    "geometry": { "type": "Point", "coordinates": [120.0, 30.0] },
                    "properties": { "name": "Hangzhou" }
                }]
            },
            "markers": [{ "longitude": 120.0, "latitude": 30.0, "label": "Hangzhou" }],
            "tile_source_id": "must-not-be-exported"
        }))
        .unwrap();

        let geojson: serde_json::Value =
            serde_json::from_slice(&map_geojson(&spec).unwrap()).unwrap();
        assert_eq!(geojson["type"], "FeatureCollection");
        assert_eq!(geojson["features"][0]["properties"]["name"], "Hangzhou");
        assert!(geojson.get("markers").is_none());
        assert!(geojson.get("tile_source_id").is_none());
    }
}
