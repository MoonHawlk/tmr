use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};
use crate::workspace::Workspace;

/// Soft limit on how large a file we'll load into memory. Large files are
/// refused with a clear error rather than silently freezing the UI.
pub const MAX_FILE_SIZE_BYTES: u64 = 10 * 1024 * 1024; // 10 MB

/// Reads a UTF-8 text file, guarded to stay within the workspace.
pub fn read_file(ws: &Workspace, path: &Path) -> Result<String> {
    let target = ws.guard(path)?;
    let meta = std::fs::metadata(&target).map_err(|e| AppError::from_io(&target, e))?;
    if meta.len() > MAX_FILE_SIZE_BYTES {
        return Err(AppError::FileTooLarge {
            path: target,
            size_mb: meta.len() / (1024 * 1024),
            limit_mb: MAX_FILE_SIZE_BYTES / (1024 * 1024),
        });
    }
    let bytes = std::fs::read(&target).map_err(|e| AppError::from_io(&target, e))?;
    String::from_utf8(bytes).map_err(|_| AppError::InvalidUtf8(target))
}

/// Writes `content` to `path` atomically: written to a sibling temp file
/// then renamed over the destination, so a crash mid-write can't corrupt
/// the original file.
pub fn save_file(ws: &Workspace, path: &Path, content: &str) -> Result<PathBuf> {
    let target = ws.guard(path)?;
    let parent = target.parent().unwrap_or_else(|| ws.root());
    let tmp_name = format!(
        ".{}.tmp",
        target.file_name().and_then(|n| n.to_str()).unwrap_or("tmr")
    );
    let tmp_path = parent.join(tmp_name);
    std::fs::write(&tmp_path, content).map_err(|e| AppError::from_io(&tmp_path, e))?;
    std::fs::rename(&tmp_path, &target).map_err(|e| AppError::from_io(&target, e))?;
    Ok(target)
}

/// Creates a new, empty file. Fails if a file already exists at that path.
pub fn create_file(ws: &Workspace, path: &Path) -> Result<PathBuf> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        ws.root().join(path)
    };
    if candidate.exists() {
        return Err(AppError::AlreadyExists(candidate));
    }
    if let Some(parent) = candidate.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::from_io(parent, e))?;
    }
    std::fs::write(&candidate, "").map_err(|e| AppError::from_io(&candidate, e))?;
    ws.guard(&candidate)
}

/// Deletes a file. Callers (the TUI) are responsible for confirming with
/// the user before calling this — the core performs no confirmation of its
/// own, it just executes the operation safely.
pub fn delete_file(ws: &Workspace, path: &Path) -> Result<()> {
    let target = ws.guard(path)?;
    let meta = std::fs::symlink_metadata(&target).map_err(|e| AppError::from_io(&target, e))?;
    if meta.file_type().is_symlink() {
        // Never follow a symlink for a destructive op; remove the link itself.
        std::fs::remove_file(&target).map_err(|e| AppError::from_io(&target, e))?;
        return Ok(());
    }
    if !meta.is_file() {
        return Err(AppError::Io {
            path: target.clone(),
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "not a regular file"),
        });
    }
    std::fs::remove_file(&target).map_err(|e| AppError::from_io(&target, e))
}

/// Renames/moves a file within the workspace.
pub fn rename_file(ws: &Workspace, from: &Path, to: &Path) -> Result<PathBuf> {
    let src = ws.guard(from)?;
    if to.exists() {
        return Err(AppError::AlreadyExists(to.to_path_buf()));
    }
    let dest = if to.is_absolute() {
        to.to_path_buf()
    } else {
        ws.root().join(to)
    };
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::from_io(parent, e))?;
    }
    std::fs::rename(&src, &dest).map_err(|e| AppError::from_io(&dest, e))?;
    ws.guard(&dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_workspace() -> (tempfile::TempDir, Workspace) {
        let dir = tempfile::tempdir().unwrap();
        let ws = Workspace::new(dir.path().to_path_buf()).unwrap();
        (dir, ws)
    }

    #[test]
    fn create_read_save_roundtrip() {
        let (_dir, ws) = make_workspace();
        let path = PathBuf::from("note.md");
        create_file(&ws, &path).unwrap();
        assert_eq!(read_file(&ws, &path).unwrap(), "");
        save_file(&ws, &path, "# Hello").unwrap();
        assert_eq!(read_file(&ws, &path).unwrap(), "# Hello");
    }

    #[test]
    fn create_fails_if_exists() {
        let (_dir, ws) = make_workspace();
        let path = PathBuf::from("note.md");
        create_file(&ws, &path).unwrap();
        assert!(matches!(
            create_file(&ws, &path),
            Err(AppError::AlreadyExists(_))
        ));
    }

    #[test]
    fn delete_removes_file() {
        let (_dir, ws) = make_workspace();
        let path = PathBuf::from("note.md");
        create_file(&ws, &path).unwrap();
        delete_file(&ws, &path).unwrap();
        assert!(matches!(read_file(&ws, &path), Err(AppError::NotFound(_))));
    }

    #[test]
    fn rename_moves_file() {
        let (_dir, ws) = make_workspace();
        let from = PathBuf::from("a.md");
        let to = PathBuf::from("b.md");
        create_file(&ws, &from).unwrap();
        save_file(&ws, &from, "content").unwrap();
        rename_file(&ws, &from, &to).unwrap();
        assert_eq!(read_file(&ws, &to).unwrap(), "content");
        assert!(matches!(read_file(&ws, &from), Err(AppError::NotFound(_))));
    }

    #[test]
    fn read_rejects_oversized_file() {
        let (_dir, ws) = make_workspace();
        let path = PathBuf::from("big.md");
        let full = ws.root().join(&path);
        let big = vec![b'a'; (MAX_FILE_SIZE_BYTES + 1) as usize];
        std::fs::write(&full, big).unwrap();
        assert!(matches!(
            read_file(&ws, &path),
            Err(AppError::FileTooLarge { .. })
        ));
    }
}
