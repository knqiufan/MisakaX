#[cfg(feature = "test-private")]
mod tests {
    use misaka_x_lib::commands::session::{export_sessions_to_file, import_sessions_from_file};
    use misaka_x_lib::config::AppConfig;
    use misaka_x_lib::db::migrations::run_migrations;
    use misaka_x_lib::db::repository::{MessageRepo, SessionRepo, WorkspaceRepo};
    use misaka_x_lib::services::llm::StreamRegistry;
    use misaka_x_lib::services::mcp::McpManager;
    use misaka_x_lib::services::sidecar_client::SidecarClient;
    use misaka_x_lib::sidecar::SidecarManager;
    use misaka_x_lib::AppState;
    use rusqlite::Connection;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    fn create_test_state() -> AppState {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        run_migrations(&conn).unwrap();

        AppState {
            db: Mutex::new(conn),
            config: Mutex::new(AppConfig::default()),
            sidecar: Arc::new(SidecarManager::new(PathBuf::from("agent"), 9527)),
            sidecar_client: SidecarClient::new(9527),
            stream_registry: StreamRegistry::new(),
            mcp_manager: Arc::new(McpManager::new()),
        }
    }

    #[test]
    fn create_session_flow_with_working_directory() {
        let state = create_test_state();
        let conn = state.db.lock().unwrap();

        let session = SessionRepo::create(
            &conn,
            "test-sess-1",
            Some("My Chat"),
            Some("config1:claude-sonnet"),
            Some("/home/user/project"),
        )
        .unwrap();

        assert_eq!(session.id, "test-sess-1");
        assert_eq!(session.title, Some("My Chat".to_string()));
        assert_eq!(session.model, Some("config1:claude-sonnet".to_string()));
        assert_eq!(
            session.working_directory,
            Some("/home/user/project".to_string())
        );
        assert_eq!(session.project_name, Some("project".to_string()));
        assert_eq!(session.status, "active");

        let id = uuid::Uuid::new_v4().to_string();
        WorkspaceRepo::record_usage(&conn, &id, "/home/user/project", Some("project")).unwrap();

        let recent = WorkspaceRepo::find_recent(&conn, 10).unwrap();
        assert!(recent.iter().any(|r| r.path == "/home/user/project"));
    }

    #[test]
    fn create_session_flow_without_working_directory() {
        let state = create_test_state();
        let conn = state.db.lock().unwrap();

        let session = SessionRepo::create(&conn, "test-sess-2", None, None, None).unwrap();

        assert_eq!(session.working_directory, None);
        assert_eq!(session.project_name, None);
    }

    #[test]
    fn update_working_directory_flow() {
        let state = create_test_state();
        let conn = state.db.lock().unwrap();

        SessionRepo::create(&conn, "test-sess-3", None, None, None).unwrap();

        SessionRepo::update_working_directory(&conn, "test-sess-3", Some("/new/workspace"))
            .unwrap();

        let session = SessionRepo::find_by_id(&conn, "test-sess-3").unwrap();
        assert_eq!(
            session.working_directory,
            Some("/new/workspace".to_string())
        );
        assert_eq!(session.project_name, Some("workspace".to_string()));
    }

    #[test]
    fn update_working_directory_clears_when_none() {
        let state = create_test_state();
        let conn = state.db.lock().unwrap();

        SessionRepo::create(&conn, "test-sess-4", None, None, Some("/initial")).unwrap();

        SessionRepo::update_working_directory(&conn, "test-sess-4", None).unwrap();

        let session = SessionRepo::find_by_id(&conn, "test-sess-4").unwrap();
        assert_eq!(session.working_directory, None);
        assert_eq!(session.project_name, None);
    }

    #[test]
    fn get_session_flow() {
        let state = create_test_state();
        let conn = state.db.lock().unwrap();

        SessionRepo::create(
            &conn,
            "test-sess-5",
            Some("Get Test"),
            None,
            Some("/path/proj"),
        )
        .unwrap();

        let session = SessionRepo::find_by_id(&conn, "test-sess-5").unwrap();
        assert_eq!(session.title, Some("Get Test".to_string()));
        assert_eq!(session.working_directory, Some("/path/proj".to_string()));
    }

    #[test]
    fn full_workspace_binding_flow() {
        let state = create_test_state();
        let conn = state.db.lock().unwrap();

        let session = SessionRepo::create(
            &conn,
            "binding-flow",
            Some("Workspace Binding"),
            None,
            Some("D:\\code\\Misaka-Tauri"),
        )
        .unwrap();
        assert_eq!(
            session.working_directory,
            Some("D:\\code\\Misaka-Tauri".to_string())
        );
        assert_eq!(session.project_name, Some("Misaka-Tauri".to_string()));

        let id = uuid::Uuid::new_v4().to_string();
        WorkspaceRepo::record_usage(&conn, &id, "D:\\code\\Misaka-Tauri", Some("Misaka-Tauri"))
            .unwrap();
        let recent = WorkspaceRepo::find_recent(&conn, 10).unwrap();
        assert!(recent.iter().any(|r| r.path == "D:\\code\\Misaka-Tauri"));

        SessionRepo::update_working_directory(
            &conn,
            "binding-flow",
            Some("C:\\Projects\\other-project"),
        )
        .unwrap();

        let updated = SessionRepo::find_by_id(&conn, "binding-flow").unwrap();
        assert_eq!(
            updated.working_directory,
            Some("C:\\Projects\\other-project".to_string())
        );
        assert_eq!(updated.project_name, Some("other-project".to_string()));
    }

    #[test]
    fn export_import_roundtrip_preserves_messages_and_tool_calls() {
        let source = create_test_state();
        let source_conn = source.db.lock().unwrap();
        SessionRepo::create(
            &source_conn,
            "roundtrip-session",
            Some("Roundtrip"),
            Some("gpt-4o"),
            Some("D:\\code\\Misaka-Tauri"),
        )
        .unwrap();
        MessageRepo::insert_user_message(
            &source_conn,
            "user-1",
            "roundtrip-session",
            "Read package.json",
            None,
        )
        .unwrap();
        MessageRepo::insert_assistant_placeholder(
            &source_conn,
            "assistant-1",
            "roundtrip-session",
            "gpt-4o",
        )
        .unwrap();
        let tool_calls = r#"[{"id":"tc1","tool_name":"read_file","status":"complete"}]"#;
        MessageRepo::update_assistant_content(
            &source_conn,
            "assistant-1",
            "Done",
            None,
            None,
            false,
            Some(tool_calls),
        )
        .unwrap();

        let dir = tempfile::tempdir().unwrap();
        let export_path = dir.path().join("sessions.json");
        export_sessions_to_file(
            &source_conn,
            &["roundtrip-session".to_string()],
            &export_path,
        )
        .unwrap();

        let target = create_test_state();
        let target_conn = target.db.lock().unwrap();
        let result = import_sessions_from_file(&target_conn, &export_path).unwrap();
        assert_eq!(result.imported_count, 1);
        assert_eq!(result.skipped_count, 0);
        assert!(result.errors.is_empty());

        let imported = SessionRepo::find_by_id(&target_conn, "roundtrip-session").unwrap();
        assert_eq!(imported.title.as_deref(), Some("Roundtrip"));
        assert_eq!(imported.project_name.as_deref(), Some("Misaka-Tauri"));

        let messages = MessageRepo::find_recent(&target_conn, "roundtrip-session", 10).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].content, "Read package.json");
        assert_eq!(messages[1].tool_calls.as_deref(), Some(tool_calls));
    }
} // mod tests
