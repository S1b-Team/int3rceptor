use crate::capture::{CaptureEntry, CaptureQuery};
use crate::error::Result;
use crate::storage::CaptureStorage;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use time::OffsetDateTime;

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub name: String,
    pub description: String,
    pub created_at: i64,
    pub modified_at: i64,
    pub version: String,
}

impl Default for ProjectInfo {
    fn default() -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Self {
            name: "Untitled Project".to_string(),
            description: String::new(),
            created_at: now,
            modified_at: now,
            version: "1.0".to_string(),
        }
    }
}

/// Complete project data for export/import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub info: ProjectInfo,
    pub traffic: Vec<CaptureEntry>,
    pub scope: Vec<String>,
    pub settings: serde_json::Value,
}

impl ProjectData {
    pub fn new(name: impl Into<String>) -> Self {
        let mut info = ProjectInfo::default();
        info.name = name.into();
        Self {
            info,
            traffic: Vec::new(),
            scope: Vec::new(),
            settings: serde_json::json!({}),
        }
    }

    /// Export project to JSON file
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Import project from JSON file
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let project: ProjectData = serde_json::from_str(&content)?;
        Ok(project)
    }

    /// Get project info summary
    pub fn summary(&self) -> ProjectSummary {
        ProjectSummary {
            name: self.info.name.clone(),
            description: self.info.description.clone(),
            traffic_count: self.traffic.len(),
            scope_count: self.scope.len(),
            created_at: self.info.created_at,
            modified_at: self.info.modified_at,
        }
    }
}

/// Lightweight project summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub name: String,
    pub description: String,
    pub traffic_count: usize,
    pub scope_count: usize,
    pub created_at: i64,
    pub modified_at: i64,
}

/// Project manager for handling project operations
pub struct ProjectManager {
    current_project: parking_lot::RwLock<ProjectInfo>,
    storage: Option<Arc<CaptureStorage>>,
}

impl ProjectManager {
    pub fn new(storage: Option<Arc<CaptureStorage>>) -> Self {
        Self {
            current_project: parking_lot::RwLock::new(ProjectInfo::default()),
            storage,
        }
    }

    /// Get current project info
    pub fn current_info(&self) -> ProjectInfo {
        self.current_project.read().clone()
    }

    /// Update project info
    pub fn update_info(&self, name: Option<String>, description: Option<String>) {
        let mut info = self.current_project.write();
        if let Some(n) = name {
            info.name = n;
        }
        if let Some(d) = description {
            info.description = d;
        }
        info.modified_at = OffsetDateTime::now_utc().unix_timestamp();
    }

    /// Export current project data
    pub fn export(&self, scope: Vec<String>, settings: serde_json::Value) -> Result<ProjectData> {
        let info = self.current_info();

        // Get all traffic from storage
        let traffic = if let Some(storage) = &self.storage {
            storage.query(&CaptureQuery::default())?
        } else {
            Vec::new()
        };

        Ok(ProjectData {
            info,
            traffic,
            scope,
            settings,
        })
    }

    /// Import project data and replace current state
    pub fn import(&self, data: ProjectData) -> Result<()> {
        // Update current project info
        {
            let mut info = self.current_project.write();
            *info = data.info.clone();
            info.modified_at = OffsetDateTime::now_utc().unix_timestamp();
        }

        // Clear and reload storage if available
        if let Some(storage) = &self.storage {
            storage.clear()?;
            for entry in &data.traffic {
                storage.insert(entry)?;
            }
        }

        Ok(())
    }

    /// Create a new empty project
    pub fn new_project(&self, name: impl Into<String>) {
        let mut info = self.current_project.write();
        *info = ProjectInfo::default();
        info.name = name.into();

        // Clear storage
        if let Some(storage) = &self.storage {
            let _ = storage.clear();
        }
    }
}

impl Default for ProjectManager {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_project_data_save_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test_project.i3p");

        let mut project = ProjectData::new("Test Project");
        project.info.description = "A test project".to_string();
        project.scope = vec!["example.com".to_string(), "*.test.com".to_string()];

        project.save_to_file(&path).unwrap();
        assert!(path.exists());

        let loaded = ProjectData::load_from_file(&path).unwrap();
        assert_eq!(loaded.info.name, "Test Project");
        assert_eq!(loaded.info.description, "A test project");
        assert_eq!(loaded.scope.len(), 2);
    }

    #[test]
    fn test_project_manager() {
        let manager = ProjectManager::new(None);

        assert_eq!(manager.current_info().name, "Untitled Project");

        manager.update_info(
            Some("My Audit".to_string()),
            Some("Security audit".to_string()),
        );

        let info = manager.current_info();
        assert_eq!(info.name, "My Audit");
        assert_eq!(info.description, "Security audit");
    }

    #[test]
    fn test_new_project() {
        let manager = ProjectManager::new(None);
        manager.update_info(Some("Old Project".to_string()), None);

        manager.new_project("Fresh Start");

        let info = manager.current_info();
        assert_eq!(info.name, "Fresh Start");
        assert!(info.description.is_empty());
    }
}
