//! Read the installed Electron app's version without launching it.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};

const MAX_HEADER: u32 = 4 * 1024 * 1024;
const MAX_PACKAGE: u64 = 64 * 1024;

#[derive(Deserialize)]
struct Header {
    files: Files,
}

#[derive(Deserialize)]
struct Files {
    #[serde(rename = "package.json")]
    package: Entry,
}

#[derive(Deserialize)]
struct Entry {
    offset: String,
    size: u64,
}

#[derive(Deserialize)]
struct Package {
    version: String,
}

fn invalid() -> Error {
    Error::Invalid("could not read Antigravity version from app.asar".into())
}

pub fn read(archive: &Path) -> Result<String> {
    read_archive(&mut File::open(archive)?)
}

fn read_archive(archive: &mut (impl Read + Seek)) -> Result<String> {
    // Electron ASAR: a size pickle, then a pickle containing the JSON header.
    // File offsets are relative to the end of both pickles (8 + header size).
    let mut prefix = [0u8; 16];
    archive.read_exact(&mut prefix)?;
    let word = |i| u32::from_le_bytes(prefix[i..i + 4].try_into().unwrap());
    let (size, payload, json_size) = (word(4), word(8), word(12));
    if word(0) != 4
        || !(8..=MAX_HEADER).contains(&size)
        || payload != size - 4
        || json_size == 0
        || json_size > size - 8
    {
        return Err(invalid());
    }
    let mut json = vec![0; json_size as usize];
    archive.read_exact(&mut json)?;
    let header: Header = serde_json::from_slice(&json)?;
    let entry = header.files.package;
    if entry.size == 0 || entry.size > MAX_PACKAGE {
        return Err(invalid());
    }
    let offset = entry
        .offset
        .parse::<u64>()
        .ok()
        .and_then(|offset| offset.checked_add(8 + u64::from(size)))
        .ok_or_else(invalid)?;
    let end = offset.checked_add(entry.size).ok_or_else(invalid)?;
    if end > archive.seek(SeekFrom::End(0))? {
        return Err(invalid());
    }
    archive.seek(SeekFrom::Start(offset))?;
    let mut json = vec![0; entry.size as usize];
    archive.read_exact(&mut json)?;
    let package: Package = serde_json::from_slice(&json)?;
    // A version is a single HTTP product token, never arbitrary header text.
    if !package.version.starts_with(|c: char| c.is_ascii_digit())
        || !package
            .version
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'-' | b'+'))
    {
        return Err(invalid());
    }
    Ok(package.version)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn archive(package: &[u8], offset: &str, size: u64) -> Vec<u8> {
        let header = serde_json::to_vec(&serde_json::json!({
            "files": {"package.json": {"offset": offset, "size": size}}
        }))
        .unwrap();
        let padded = (header.len() + 3) & !3;
        let header_size = (8 + padded) as u32;
        let mut bytes = Vec::new();
        for word in [4, header_size, header_size - 4, header.len() as u32] {
            bytes.extend(word.to_le_bytes());
        }
        bytes.extend(header);
        bytes.resize(8 + header_size as usize, 0);
        bytes.extend(b"other file");
        bytes.extend(package);
        bytes
    }

    #[test]
    fn reads_package_at_its_archive_offset() {
        let package = br#"{"version":"2.23.1"}"#;
        let bytes = archive(package, "10", package.len() as u64);
        assert_eq!(read_archive(&mut Cursor::new(bytes)).unwrap(), "2.23.1");
    }

    #[test]
    fn rejects_invalid_header_and_entry_bounds() {
        for (offset, size) in [
            ("10", MAX_PACKAGE + 1),
            ("999", 1),
            ("18446744073709551615", 1),
        ] {
            assert!(read_archive(&mut Cursor::new(archive(b"", offset, size))).is_err());
        }
        let mut bytes = archive(b"", "10", 1);
        bytes[4..8].copy_from_slice(&(MAX_HEADER + 1).to_le_bytes());
        assert!(read_archive(&mut Cursor::new(bytes)).is_err());
        assert!(read_archive(&mut Cursor::new([0; 8])).is_err());
    }

    #[test]
    fn rejects_missing_or_unsafe_versions() {
        for package in [
            br#"{}"#.as_slice(),
            br#"{"version":""}"#,
            br#"{"version":"2.23.1\r\nOther: value"}"#,
        ] {
            assert!(
                read_archive(&mut Cursor::new(archive(
                    package,
                    "10",
                    package.len() as u64
                )))
                .is_err()
            );
        }
    }
}
