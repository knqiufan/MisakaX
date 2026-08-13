use misaka_x_lib::db::migrations::run_migrations;
use misaka_x_lib::db::repository::{ProfileRepo, SettingsRepo};
use rusqlite::Connection;

fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
    run_migrations(&conn).unwrap();
    conn
}

#[test]
fn default_profile_is_created_once_and_selected() {
    let conn = setup();
    let first = ProfileRepo::create_default(&conn).unwrap();
    let second = ProfileRepo::create_default(&conn).unwrap();
    let current = ProfileRepo::get_current(&conn).unwrap();

    assert_eq!(first.profile_id, second.profile_id);
    assert_eq!(first, current);
    assert_eq!(current.profile_kind, "local");
    assert_eq!(current.timezone_mode, "system");
    assert!(current.timezone_id.is_none());

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM user_profiles", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn ensure_default_repairs_a_dangling_current_profile_setting() {
    let conn = setup();
    let profile = ProfileRepo::create_default(&conn).unwrap();
    SettingsRepo::set(&conn, "profile.current_id", "missing-profile").unwrap();

    let repaired = ProfileRepo::ensure_default(&conn).unwrap();
    assert_eq!(repaired.profile_id, profile.profile_id);
    assert_eq!(ProfileRepo::get_current(&conn).unwrap(), profile);
}

#[test]
fn profile_name_avatar_and_timezone_updates_round_trip() {
    let conn = setup();
    let profile = ProfileRepo::create_default(&conn).unwrap();

    let named =
        ProfileRepo::update_display_name(&conn, &profile.profile_id, "Misaka User").unwrap();
    assert_eq!(named.display_name, "Misaka User");

    let avatar =
        ProfileRepo::update_avatar(&conn, &profile.profile_id, "avatars/profile.webp", "abc123")
            .unwrap();
    assert_eq!(
        avatar.avatar_storage_key.as_deref(),
        Some("avatars/profile.webp")
    );
    assert_eq!(avatar.avatar_sha256.as_deref(), Some("abc123"));

    let timezone =
        ProfileRepo::update_timezone_snapshot(&conn, &profile.profile_id, Some("Asia/Shanghai"), 1)
            .unwrap();
    assert_eq!(timezone.timezone_id.as_deref(), Some("Asia/Shanghai"));
    assert_eq!(timezone.week_start, 1);

    let cleared = ProfileRepo::clear_avatar(&conn, &profile.profile_id).unwrap();
    assert!(cleared.avatar_storage_key.is_none());
    assert!(cleared.avatar_sha256.is_none());
}

#[test]
fn profile_constraints_reject_blank_long_names_and_invalid_week_start() {
    let conn = setup();
    let profile = ProfileRepo::create_default(&conn).unwrap();
    assert!(ProfileRepo::update_display_name(&conn, &profile.profile_id, "   ").is_err());
    assert!(ProfileRepo::update_display_name(&conn, &profile.profile_id, &"x".repeat(41)).is_err());
    assert!(ProfileRepo::update_timezone_snapshot(&conn, &profile.profile_id, None, 2).is_err());
}
