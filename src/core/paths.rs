//! XDG-compliant path resolution for schemas, config, and project roots.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

const DATA_SUBDIR: &str = "specify";
const PROJECT_DIR: &str = ".specify";
const LEGACY_PROJECT_DIR: &str = "specify";

/// Resolved paths for the Specify data directory.
#[derive(Debug, Clone)]
pub struct DataDir {
    root: PathBuf,
}

impl DataDir {
    /// Resolve the global data directory (`~/.local/share/specify` on Linux/macOS).
    ///
    /// # Errors
    ///
    /// Returns an error if the platform data directory cannot be determined.
    pub fn resolve() -> Result<Self> {
        let base = dirs::data_dir().context("unable to determine platform data directory")?;
        Ok(Self {
            root: base.join(DATA_SUBDIR),
        })
    }

    /// Path to the global schemas directory.
    #[must_use]
    pub fn schemas_dir(&self) -> PathBuf {
        self.root.join("schemas")
    }

    /// Path to a specific schema directory within the global store.
    #[must_use]
    pub fn schema_dir(&self, name: &str) -> PathBuf {
        self.schemas_dir().join(name)
    }

    /// Root of the data directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Ensure the data directory tree exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    pub fn ensure(&self) -> Result<()> {
        std::fs::create_dir_all(self.schemas_dir())
            .with_context(|| format!("creating data directory at {}", self.root.display()))
    }
}

/// Resolved paths for a project's `.specify/` directory.
#[derive(Debug, Clone)]
pub struct ProjectDir {
    root: PathBuf,
}

impl ProjectDir {
    /// Construct from an explicit project root (the parent of `.specify/`).
    #[must_use]
    pub fn from_root(project_root: &Path) -> Self {
        Self {
            root: project_root.join(PROJECT_DIR),
        }
    }

    /// Locate the project's `.specify/` directory by searching upward from `start`.
    ///
    /// Checks for `.specify/` first, then falls back to the legacy `specify/`
    /// directory for backward compatibility.
    ///
    /// # Errors
    ///
    /// Returns an error if no `.specify/` or `specify/` directory is found.
    pub fn discover(start: &Path) -> Result<Self> {
        let mut current = start.to_path_buf();
        loop {
            let preferred = current.join(PROJECT_DIR);
            if preferred.is_dir() {
                return Ok(Self { root: preferred });
            }
            let legacy = current.join(LEGACY_PROJECT_DIR);
            if legacy.is_dir() {
                return Ok(Self { root: legacy });
            }
            if !current.pop() {
                bail!("no .specify/ directory found (searched upward from {})", start.display());
            }
        }
    }

    /// Root of the specify directory (`<project>/.specify/`).
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Path to `.specify/config.yaml`.
    #[must_use]
    pub fn config_file(&self) -> PathBuf {
        self.root.join("config.yaml")
    }

    /// Path to the project's schemas directory.
    #[must_use]
    pub fn schemas_dir(&self) -> PathBuf {
        self.root.join("schemas")
    }

    /// Path to a specific schema within the project.
    #[must_use]
    pub fn schema_dir(&self, name: &str) -> PathBuf {
        self.schemas_dir().join(name)
    }

    /// Path to the changes directory.
    #[must_use]
    pub fn changes_dir(&self) -> PathBuf {
        self.root.join("changes")
    }

    /// Path to a specific change directory.
    #[must_use]
    pub fn change_dir(&self, name: &str) -> PathBuf {
        self.changes_dir().join(name)
    }

    /// Path to the baseline specs directory (`.specify/specs/`).
    #[must_use]
    pub fn specs_dir(&self) -> PathBuf {
        self.root.join("specs")
    }

    /// Path to a specific capability's spec directory (`.specify/specs/<capability>/`).
    #[must_use]
    pub fn spec_dir(&self, capability: &str) -> PathBuf {
        self.specs_dir().join(capability)
    }

    /// Path to the change archive directory (`.specify/changes/archive/`).
    #[must_use]
    pub fn archive_dir(&self) -> PathBuf {
        self.changes_dir().join("archive")
    }

    /// Whether the `.specify` directory already exists.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.root.is_dir()
    }

    /// Ensure the full directory skeleton exists (changes/ and specs/).
    ///
    /// # Errors
    ///
    /// Returns an error if the directories cannot be created.
    pub fn ensure(&self) -> Result<()> {
        std::fs::create_dir_all(self.changes_dir())
            .with_context(|| format!("creating {}", self.changes_dir().display()))?;
        std::fs::create_dir_all(self.specs_dir())
            .with_context(|| format!("creating {}", self.specs_dir().display()))
    }
}
