use std::fs::File;
use std::io;
use std::path::Path;

use anyhow::{bail, Context, Result};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

pub fn export_skill_dir(skill_dir: &Path, destination: &Path) -> Result<()> {
    if !skill_dir.join("SKILL.md").is_file() {
        bail!("Installed skill is missing SKILL.md")
    }
    let output = File::create(destination).context("Cannot create skills export file")?;
    let mut writer = ZipWriter::new(output);
    for entry in WalkDir::new(skill_dir).follow_links(false) {
        write_entry(&mut writer, skill_dir, entry?)?;
    }
    writer.finish().context("Cannot finalize skills export")?;
    Ok(())
}

fn write_entry(writer: &mut ZipWriter<File>, root: &Path, entry: walkdir::DirEntry) -> Result<()> {
    let path = entry.path();
    if path == root {
        return Ok(());
    }
    if entry.file_type().is_symlink() {
        bail!("Cannot export a skill that contains symbolic links")
    }
    if !entry.file_type().is_file() {
        return Ok(());
    }

    let relative = path
        .strip_prefix(root)
        .context("Skill export path is outside its root")?;
    let name = relative
        .to_str()
        .context("Skill export path is not valid UTF-8")?
        .replace('\\', "/");
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    writer
        .start_file(name, options)
        .context("Cannot add skills export file")?;
    let mut input = File::open(path).context("Cannot read skill export file")?;
    io::copy(&mut input, writer).context("Cannot write skill export file")?;
    Ok(())
}
