use super::*;

use acme_kit::text;

/// Where a data export manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DataExportManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DataExportManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DataExportManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One data export manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportManifest {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub principal: String,
    pub cluster: String,
    pub category: String,
    pub status: DataExportManifestStatus,
}

impl DataExportManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DataExportManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DataExportManifestStatus) -> bool {
        match self.status {
            DataExportManifestStatus::Created => {
                match next {
                    DataExportManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DataExportManifestStatus::Validated => {
                return next == DataExportManifestStatus::Published;
            }
            DataExportManifestStatus::Published => {
                return next == DataExportManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DataExportManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "data export manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "data export manifest", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> DataExportManifest {
        DataExportManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            principal: self.principal.clone(),
            cluster: self.cluster.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a data export manifest in status created.
pub fn new_data_export_manifest(code: &str, name: &str) -> DataExportManifest {
    DataExportManifest {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        principal: "".to_string(),
        cluster: "".to_string(),
        category: "".to_string(),
        status: DataExportManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_data_export_manifest("code-555", "name-539");
        assert_eq!(item.status, DataExportManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_data_export_manifest("code-555", "name-539");
        if item.move_to(DataExportManifestStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DataExportManifestStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_data_export_manifest("code-555", "name-539");
        item.status = DataExportManifestStatus::Retired;
        assert!(
            item.move_to(DataExportManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
