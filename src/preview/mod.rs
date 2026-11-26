mod highlight;

pub(crate) use highlight::Token;
pub(crate) use highlight::prepare;

use std::{
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

const MAX_BYTES: usize = 1024 * 1024;

pub(crate) struct Document {
    pub(crate) text: String,
    pub(crate) truncated: bool,
    pub(crate) highlights: Option<highlight::Highlights>,
}

pub(crate) enum Error {
    Read(io::Error),
    NotRegular,
    Binary,
    Encoding,
}

impl From<io::Error> for Error {
    fn from(cause: io::Error) -> Self {
        Self::Read(cause)
    }
}

pub(crate) fn open(path: &Path) -> Result<Document, Error> {
    let file = open_regular(path)?;
    let mut bytes = Vec::new();
    file.take((MAX_BYTES + 1) as u64).read_to_end(&mut bytes)?;
    let truncated = bytes.len() > MAX_BYTES;
    bytes.truncate(MAX_BYTES);
    let text = decode(&bytes, truncated)?;
    let highlights = highlight::highlight(path, &text);
    Ok(Document {
        text,
        truncated,
        highlights,
    })
}

pub(crate) fn full_text(path: &Path, document: &Document) -> Result<String, Error> {
    if !document.truncated {
        return Ok(document.text.clone());
    }
    let mut bytes = Vec::new();
    open_regular(path)?.read_to_end(&mut bytes)?;
    decode(&bytes, false)
}

fn open_regular(path: &Path) -> Result<File, Error> {
    if !fs::metadata(path)?.is_file() {
        return Err(Error::NotRegular);
    }
    let file = File::open(path)?;
    if !file.metadata()?.is_file() {
        return Err(Error::NotRegular);
    }
    Ok(file)
}

fn decode(bytes: &[u8], truncated: bool) -> Result<String, Error> {
    let text = match bytes {
        [0xff, 0xfe, 0, 0, ..] | [0, 0, 0xfe, 0xff, ..] => return Err(Error::Encoding),
        [0xff, 0xfe, rest @ ..] => decode_utf16(rest, u16::from_le_bytes, truncated)?,
        [0xfe, 0xff, rest @ ..] => decode_utf16(rest, u16::from_be_bytes, truncated)?,
        _ => {
            let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
            if bytes
                .iter()
                .any(|byte| matches!(byte, 0..=8 | 11..=12 | 14..=31 | 127))
            {
                return Err(Error::Binary);
            }
            match str::from_utf8(bytes) {
                Ok(text) => text.to_owned(),
                Err(error) if truncated && error.error_len().is_none() => {
                    str::from_utf8(&bytes[..error.valid_up_to()])
                        .map_err(|_| Error::Encoding)?
                        .to_owned()
                }
                Err(_) => return Err(Error::Encoding),
            }
        }
    };
    if text.contains('\0') {
        return Err(Error::Binary);
    }
    Ok(text)
}

fn decode_utf16(
    bytes: &[u8],
    read_unit: fn([u8; 2]) -> u16,
    truncated: bool,
) -> Result<String, Error> {
    let (pairs, remainder) = bytes.as_chunks::<2>();
    if !truncated && !remainder.is_empty() {
        return Err(Error::Encoding);
    }
    let mut units: Vec<_> = pairs.iter().copied().map(read_unit).collect();
    if truncated && matches!(units.last(), Some(0xd800..=0xdbff)) {
        units.pop();
    }
    String::from_utf16(&units).map_err(|_| Error::Encoding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_boms_and_handles_incomplete_unicode_at_the_limit() {
        assert_eq!(
            decode(b"\xef\xbb\xbftext", false).ok().as_deref(),
            Some("text")
        );
        assert_eq!(
            decode(&[0xff, 0xfe, b'a', 0], false).ok().as_deref(),
            Some("a")
        );
        assert_eq!(decode(b"a\xe2\x82", true).ok().as_deref(), Some("a"));
        assert!(matches!(decode(b"a\xe2\x82", false), Err(Error::Encoding)));
        let partial_utf16 = [0xff, 0xfe, b'a', 0, 0x3d, 0xd8];
        assert_eq!(decode(&partial_utf16, true).ok().as_deref(), Some("a"));
        assert!(matches!(
            decode(&partial_utf16, false),
            Err(Error::Encoding)
        ));
    }

    #[test]
    fn copies_text_beyond_the_preview_limit() {
        let path =
            std::env::temp_dir().join(format!("filens-full-copy-{}.txt", std::process::id()));
        let text = format!("{}\r\nend\n", "x".repeat(MAX_BYTES));
        fs::write(&path, &text).unwrap();
        let document = open(&path).ok().unwrap();
        assert!(document.truncated);
        let copied = full_text(&path, &document);
        fs::remove_file(path).unwrap();
        assert_eq!(copied.ok().as_deref(), Some(text.as_str()));
    }

    #[cfg(unix)]
    #[test]
    fn follows_only_links_to_regular_files() {
        let path = std::env::temp_dir().join(format!("filens-preview-link-{}", std::process::id()));
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        std::os::unix::fs::symlink(root, &path).unwrap();
        assert!(matches!(open(&path), Err(Error::NotRegular)));
        fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(root.join("Cargo.toml"), &path).unwrap();
        assert!(open(&path).is_ok());
        fs::remove_file(path).unwrap();
    }
}
