use misaka_x_lib::commands::workspace::{validate_directory};
use std::fs;

#[test]
fn validate_existing_directory() {
    let temp = std::env::temp_dir().join("misaka_test_validate_exists");
    let _ = fs::create_dir_all(&temp);

    let info = validate_directory(temp.to_string_lossy().to_string()).unwrap();
    assert!(info.exists);
    assert!(info.readable);
    assert!(info.writable);
    assert!(info.file_count.is_some());
    assert!(!info.name.is_empty());

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn validate_nonexistent_directory() {
    let path = "/this/path/does/not/exist/at/all_misaka_test";
    let info = validate_directory(path.to_string()).unwrap();
    assert!(!info.exists);
    assert!(!info.readable);
    assert!(!info.writable);
    assert!(info.file_count.is_none());
}

#[test]
fn validate_directory_name_extraction() {
    let temp = std::env::temp_dir().join("misaka_test_name_extract");
    let _ = fs::create_dir_all(&temp);

    let info = validate_directory(temp.to_string_lossy().to_string()).unwrap();
    assert_eq!(info.name, "misaka_test_name_extract");

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn validate_directory_counts_entries() {
    let temp = std::env::temp_dir().join("misaka_test_count_entries");
    let _ = fs::remove_dir_all(&temp);
    let _ = fs::create_dir_all(&temp);

    fs::write(temp.join("file1.txt"), b"a").unwrap();
    fs::write(temp.join("file2.txt"), b"b").unwrap();
    fs::create_dir(temp.join("subdir")).unwrap();

    let info = validate_directory(temp.to_string_lossy().to_string()).unwrap();
    assert_eq!(info.file_count, Some(3));

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn validate_file_path_is_not_directory() {
    let temp = std::env::temp_dir().join("misaka_test_file_not_dir.txt");
    fs::write(&temp, b"content").unwrap();

    let info = validate_directory(temp.to_string_lossy().to_string()).unwrap();
    assert!(!info.exists);

    let _ = fs::remove_file(&temp);
}

#[test]
fn validate_empty_directory() {
    let temp = std::env::temp_dir().join("misaka_test_empty_dir");
    let _ = fs::remove_dir_all(&temp);
    let _ = fs::create_dir_all(&temp);

    let info = validate_directory(temp.to_string_lossy().to_string()).unwrap();
    assert!(info.exists);
    assert!(info.readable);
    assert_eq!(info.file_count, Some(0));

    let _ = fs::remove_dir_all(&temp);
}
