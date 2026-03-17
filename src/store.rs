use std::fs;

use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};

use crate::model::WorkspaceSnapshot;
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: Utf8PathBuf,
    pub workspaces: Utf8PathBuf,
    pub config: Utf8PathBuf,
    pub log: Utf8PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_open: String,
    pub confirm_overwrite: bool,
    pub auto_prompt_labels: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_open: "tui".into(),
            confirm_overwrite: true,
            auto_prompt_labels: true,
        }
    }
}

impl AppPaths {
    pub fn discover() -> Result<Self> {
        let base = dirs::home_dir()
            .ok_or_else(|| Error::Message("could not resolve home directory".into()))?
            .join("Library/Application Support/ghost-in-a-shell");
        Self::from_root(
            Utf8PathBuf::from_path_buf(base).map_err(|_| {
                Error::Message("application support path was not valid UTF-8".into())
            })?,
        )
    }

    pub fn from_root(root: Utf8PathBuf) -> Result<Self> {
        let workspaces = root.join("workspaces");
        let logs = root.join("logs");
        fs::create_dir_all(&workspaces)?;
        fs::create_dir_all(&logs)?;

        let paths = Self {
            root: root.clone(),
            workspaces,
            config: root.join("config.json"),
            log: logs.join("app.log"),
        };

        if !paths.config.exists() {
            let config = serde_json::to_string_pretty(&Config::default())?;
            fs::write(&paths.config, format!("{config}\n"))?;
        }

        Ok(paths)
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotStore {
    paths: AppPaths,
}

impl SnapshotStore {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub fn list(&self) -> Result<Vec<WorkspaceSnapshot>> {
        let mut snapshots = Vec::new();
        for entry in fs::read_dir(&self.paths.workspaces)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let path = entry.path();
                if path.extension().and_then(|extension| extension.to_str()) == Some("json") {
                    let raw = fs::read_to_string(path)?;
                    let snapshot = serde_json::from_str::<WorkspaceSnapshot>(&raw)?;
                    snapshots.push(snapshot);
                }
            }
        }

        snapshots.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        Ok(snapshots)
    }

    pub fn load(&self, name_or_slug: &str) -> Result<WorkspaceSnapshot> {
        let slug = normalize_slug(name_or_slug)?;
        let path = self.snapshot_path(&slug);
        if !path.exists() {
            return Err(Error::SnapshotNotFound(name_or_slug.into()));
        }

        let raw = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&raw)?)
    }

    pub fn save(&self, snapshot: &WorkspaceSnapshot, force: bool) -> Result<()> {
        let path = self.snapshot_path(&snapshot.slug);
        if path.exists() && !force {
            return Err(Error::SnapshotExists(snapshot.name.clone()));
        }
        self.write_snapshot(&path, snapshot)
    }

    pub fn rename(&self, old: &str, new: &str) -> Result<WorkspaceSnapshot> {
        let mut snapshot = self.load(old)?;
        let new_slug = normalize_slug(new)?;
        let new_path = self.snapshot_path(&new_slug);
        if new_path.exists() {
            return Err(Error::SnapshotExists(new.into()));
        }

        let old_path = self.snapshot_path(&snapshot.slug);
        snapshot.name = new.to_string();
        snapshot.slug = new_slug;
        snapshot.updated_at = chrono::Utc::now();
        self.write_snapshot(&new_path, &snapshot)?;
        if old_path.exists() {
            fs::remove_file(old_path)?;
        }
        Ok(snapshot)
    }

    pub fn delete(&self, name_or_slug: &str) -> Result<()> {
        let slug = normalize_slug(name_or_slug)?;
        let path = self.snapshot_path(&slug);
        if !path.exists() {
            return Err(Error::SnapshotNotFound(name_or_slug.into()));
        }
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn snapshot_path(&self, slug: &str) -> Utf8PathBuf {
        self.paths.workspaces.join(format!("{slug}.json"))
    }

    fn write_snapshot(&self, path: &Utf8Path, snapshot: &WorkspaceSnapshot) -> Result<()> {
        let serialized = serde_json::to_string_pretty(snapshot)?;
        let temp_path = path.with_extension(format!("json.tmp-{}", uuid::Uuid::new_v4().simple()));
        fs::write(&temp_path, format!("{serialized}\n"))?;
        fs::rename(temp_path, path)?;
        Ok(())
    }
}

pub fn normalize_slug(name: &str) -> Result<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(Error::InvalidSnapshotName(name.into()));
    }

    let mut slug = String::new();
    let mut last_was_dash = false;
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        return Err(Error::InvalidSnapshotName(name.into()));
    }
    Ok(slug)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use tempfile::tempdir;

    use crate::model::{TerminalSnapshot, WindowSnapshot, WorkspaceSnapshot};

    use super::*;

    fn temp_store() -> SnapshotStore {
        let temp = tempdir().unwrap();
        let root = Utf8PathBuf::from_path_buf(temp.path().to_path_buf()).unwrap();
        let paths = AppPaths::from_root(root).unwrap();
        std::mem::forget(temp);
        SnapshotStore::new(paths)
    }

    fn sample_snapshot(name: &str) -> WorkspaceSnapshot {
        WorkspaceSnapshot::new(
            name.into(),
            normalize_slug(name).unwrap(),
            vec![WindowSnapshot {
                window_index: 0,
                tabs: Vec::new(),
            }],
            vec![TerminalSnapshot::new(
                Some("Shell".into()),
                None,
                Some("/tmp".into()),
                Some("zsh".into()),
                None,
            )],
        )
    }

    #[test]
    fn normalizes_slugs() {
        assert_eq!(normalize_slug("Frontend API").unwrap(), "frontend-api");
        assert!(normalize_slug("   ").is_err());
    }

    #[test]
    fn refuses_overwrite_without_force() {
        let store = temp_store();
        let snapshot = sample_snapshot("Frontend");
        store.save(&snapshot, false).unwrap();
        assert!(matches!(
            store.save(&snapshot, false),
            Err(Error::SnapshotExists(_))
        ));
    }

    #[test]
    fn sorts_by_updated_at_descending() {
        let store = temp_store();
        let mut older = sample_snapshot("Older");
        older.updated_at = Utc::now() - Duration::days(1);
        let mut newer = sample_snapshot("Newer");
        newer.updated_at = Utc::now();
        store.save(&older, false).unwrap();
        store.save(&newer, false).unwrap();

        let listed = store.list().unwrap();
        assert_eq!(listed[0].name, "Newer");
        assert_eq!(listed[1].name, "Older");
    }

    #[test]
    fn round_trips_json() {
        let snapshot = sample_snapshot("Round Trip");
        let encoded = serde_json::to_string(&snapshot).unwrap();
        let decoded: WorkspaceSnapshot = serde_json::from_str(&encoded).unwrap();
        assert_eq!(snapshot, decoded);
    }

    #[test]
    fn renames_and_removes_old_file() {
        let store = temp_store();
        let snapshot = sample_snapshot("One");
        store.save(&snapshot, false).unwrap();
        let renamed = store.rename("One", "Two").unwrap();
        assert_eq!(renamed.slug, "two");
        assert!(store.load("Two").is_ok());
        assert!(matches!(store.load("One"), Err(Error::SnapshotNotFound(_))));
    }
}
