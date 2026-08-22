use std::path::{Path, PathBuf};

use crate::error::{AppError, Result};

/// One entry in a directory listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

/// The root directory tmr was launched against. All file operations are
/// checked against this root so the TUI can't accidentally (or via a
/// crafted `..` path) touch files outside the workspace.
#[derive(Debug, Clone)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Canonicalizes `root` so later containment checks are reliable.
    pub fn new(root: PathBuf) -> Result<Self> {
        let canonical = root
            .canonicalize()
            .map_err(|e| AppError::from_io(root.clone(), e))?;
        if !canonical.is_dir() {
            return Err(AppError::NotFound(canonical));
        }
        Ok(Workspace { root: canonical })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Ensures `path` (once canonicalized) lives inside the workspace root.
    /// Returns the canonicalized path on success.
    pub fn guard(&self, path: &Path) -> Result<PathBuf> {
        let candidate = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        // The path may not exist yet (e.g. creating a new file), so we
        // canonicalize the parent directory and re-attach the file name.
        let (base, tail) = if candidate.exists() {
            (candidate.clone(), None)
        } else {
            let file_name = candidate.file_name().map(|n| n.to_os_string());
            let parent = candidate
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| self.root.clone());
            (parent, file_name)
        };

        let canonical_base = base
            .canonicalize()
            .map_err(|e| AppError::from_io(base.clone(), e))?;

        let full = match tail {
            Some(name) => canonical_base.join(name),
            None => canonical_base,
        };

        if !full.starts_with(&self.root) {
            return Err(AppError::OutsideWorkspace(full));
        }
        Ok(full)
    }

    /// Lists a single directory (non-recursive) inside the workspace,
    /// directories first, then files, both alphabetically. Hidden entries
    /// (dotfiles) are skipped.
    pub fn list_dir(&self, dir: &Path) -> Result<Vec<Entry>> {
        let target = self.guard(dir)?;
        let read_dir = std::fs::read_dir(&target).map_err(|e| AppError::from_io(&target, e))?;

        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for entry in read_dir {
            let entry = entry.map_err(|e| AppError::from_io(&target, e))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(_) => continue,
            };
            let item = Entry {
                name,
                path: entry.path(),
                is_dir: file_type.is_dir(),
            };
            if item.is_dir {
                dirs.push(item);
            } else {
                files.push(item);
            }
        }
        dirs.sort_by_key(|e| e.name.to_lowercase());
        files.sort_by_key(|e| e.name.to_lowercase());
        dirs.extend(files);
        Ok(dirs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_workspace() -> (tempfile::TempDir, Workspace) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("b.md"), "b").unwrap();
        std::fs::write(dir.path().join("a.md"), "a").unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join(".hidden.md"), "h").unwrap();
        let ws = Workspace::new(dir.path().to_path_buf()).unwrap();
        (dir, ws)
    }

    #[test]
    fn lists_dirs_before_files_and_skips_hidden() {
        let (_dir, ws) = make_workspace();
        let entries = ws.list_dir(ws.root().to_path_buf().as_path()).unwrap();
        let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["sub", "a.md", "b.md"]);
    }

    #[test]
    fn guard_rejects_escaping_paths() {
        let (dir, ws) = make_workspace();
        let outside = dir.path().parent().unwrap().join("outside.md");
        std::fs::write(&outside, "x").ok();
        let err = ws.guard(&outside);
        assert!(matches!(err, Err(AppError::OutsideWorkspace(_))));
    }

    #[test]
    fn guard_allows_new_file_inside_workspace() {
        let (_dir, ws) = make_workspace();
        let new_path = ws.root().join("new.md");
        assert!(ws.guard(&new_path).is_ok());
    }
}
