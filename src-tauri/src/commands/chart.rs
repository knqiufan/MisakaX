use rusqlite::params;
use tauri::State;

use crate::config;
use crate::services::artifacts::{
    ArtifactMetadata, ArtifactOrigin, ArtifactService, ContentSafetyPolicy,
};
use crate::services::content::ChartSpecV1;
use crate::AppState;

fn service() -> Result<ArtifactService, String> {
    ArtifactService::new(
        config::artifacts_dir().map_err(|error| error.to_string())?,
        ContentSafetyPolicy::default(),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn chart_export_csv(
    state: State<'_, AppState>,
    session_id: String,
    message_id: String,
    spec: ChartSpecV1,
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
            "chart-data.csv".to_string(),
            "text/csv".to_string(),
            &chart_csv(&spec).into_bytes(),
        )
        .map_err(|error| error.to_string())
}

fn chart_csv(spec: &ChartSpecV1) -> String {
    let mut output = String::from("series,x,y\n");
    for series in &spec.series {
        for datum in &series.values {
            output.push_str(&csv_field(&series.name));
            output.push(',');
            output.push_str(&csv_field(&datum.x));
            output.push(',');
            output.push_str(&datum.y.to_string());
            output.push('\n');
        }
    }
    output
}

fn csv_field(value: &str) -> String {
    let value = if matches!(value.chars().next(), Some('=' | '+' | '-' | '@')) {
        format!("'{value}")
    } else {
        value.to_owned()
    };
    format!("\"{}\"", value.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::{chart_csv, ChartSpecV1};

    #[test]
    fn csv_export_neutralizes_spreadsheet_formulas() {
        let spec: ChartSpecV1 = serde_json::from_value(serde_json::json!({
            "chart_type": "line",
            "title": "Sales",
            "series": [{
                "name": "=danger",
                "values": [{ "x": "+injection", "y": 12.0 }]
            }]
        }))
        .unwrap();

        assert_eq!(
            chart_csv(&spec),
            "series,x,y\n\"'=danger\",\"'+injection\",12\n"
        );
    }
}
