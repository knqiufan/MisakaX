use misaka_x_lib::commands::fs_explorer::{
    read_text_file_inner, validate_under_root, write_text_file_inner,
};

#[test]
fn validate_accepts_path_inside_root() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("notes.md");

    assert!(validate_under_root(dir.path(), &target).is_ok());
}

#[test]
fn validate_rejects_path_outside_root() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap().path().join("notes.md");

    assert!(validate_under_root(root.path(), &outside).is_err());
}

#[test]
fn read_text_file_rejects_binary_content() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("bin.dat");
    std::fs::write(&file, [0_u8, 1, 2, 3]).unwrap();

    let err = read_text_file_inner(dir.path(), &file).unwrap_err();

    assert!(err.to_lowercase().contains("binary"));
}

#[test]
fn read_text_file_rejects_large_content() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("big.txt");
    std::fs::write(&file, vec![b'a'; 6 * 1024 * 1024]).unwrap();

    let err = read_text_file_inner(dir.path(), &file).unwrap_err();

    assert!(err.to_lowercase().contains("too large"));
}

#[test]
fn write_text_file_updates_file_inside_root() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.md");
    std::fs::write(&file, "old").unwrap();

    write_text_file_inner(dir.path(), &file, "new").unwrap();

    assert_eq!(std::fs::read_to_string(&file).unwrap(), "new");
}
