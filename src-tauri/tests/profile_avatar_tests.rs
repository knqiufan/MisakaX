use std::collections::HashSet;
use std::fs;

use image::{GenericImageView, ImageBuffer, ImageFormat, Rgba};
use misaka_x_lib::services::profile_avatar::{
    avatar_path, cleanup_orphaned_avatars, read_avatar_data_url, store_avatar, MAX_AVATAR_BYTES,
    MAX_AVATAR_DIMENSION,
};

#[test]
fn avatar_is_magic_checked_resized_reencoded_and_source_is_untouched() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source.not-an-extension");
    let storage = directory.path().join("storage");
    let image = ImageBuffer::from_pixel(800, 400, Rgba([20_u8, 40, 60, 255]));
    image.save_with_format(&source, ImageFormat::Png).unwrap();
    let original = fs::read(&source).unwrap();

    let stored = store_avatar(&source, &storage).unwrap();
    assert_eq!(fs::read(&source).unwrap(), original);
    assert_eq!(stored.sha256.len(), 64);
    assert!(stored.storage_key.ends_with(".webp"));
    let normalized = image::open(avatar_path(&storage, &stored.storage_key).unwrap()).unwrap();
    assert_eq!(normalized.dimensions(), (MAX_AVATAR_DIMENSION, 256));
    assert!(read_avatar_data_url(&storage, &stored.storage_key)
        .unwrap()
        .starts_with("data:image/webp;base64,"));
}

#[test]
fn avatar_rejects_invalid_and_oversized_input_without_touching_existing_copy() {
    let directory = tempfile::tempdir().unwrap();
    let storage = directory.path().join("storage");
    let valid = directory.path().join("valid.png");
    ImageBuffer::from_pixel(2, 2, Rgba([1_u8, 2, 3, 255]))
        .save_with_format(&valid, ImageFormat::Png)
        .unwrap();
    let stored = store_avatar(&valid, &storage).unwrap();
    let existing = fs::read(avatar_path(&storage, &stored.storage_key).unwrap()).unwrap();

    let invalid = directory.path().join("invalid.png");
    fs::write(&invalid, b"not an image").unwrap();
    assert!(store_avatar(&invalid, &storage).is_err());
    let oversized = directory.path().join("oversized.webp");
    let file = fs::File::create(&oversized).unwrap();
    file.set_len(MAX_AVATAR_BYTES + 1).unwrap();
    assert!(store_avatar(&oversized, &storage).is_err());
    assert_eq!(
        fs::read(avatar_path(&storage, &stored.storage_key).unwrap()).unwrap(),
        existing
    );
}

#[test]
fn orphan_cleanup_stays_inside_storage_and_rejects_traversal_keys() {
    let directory = tempfile::tempdir().unwrap();
    let storage = directory.path().join("storage");
    fs::create_dir_all(&storage).unwrap();
    fs::write(storage.join("avatar-active.webp"), b"active").unwrap();
    fs::write(storage.join("avatar-orphan.webp"), b"orphan").unwrap();
    fs::write(storage.join(".avatar-crashed.tmp"), b"temporary").unwrap();
    fs::write(storage.join("unrelated.txt"), b"keep").unwrap();

    let active = HashSet::from(["avatar-active.webp".to_string()]);
    assert_eq!(cleanup_orphaned_avatars(&storage, &active).unwrap(), 2);
    assert!(storage.join("avatar-active.webp").exists());
    assert!(storage.join("unrelated.txt").exists());
    assert!(avatar_path(&storage, "../outside.webp").is_err());
    assert!(avatar_path(&storage, "nested/avatar.webp").is_err());
}
