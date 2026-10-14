use super::*;

use acme_kit::text;

/// Where a migration manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl MigrationManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for MigrationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationManifest {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub starts_at: i64,
    pub margin: f64,
    pub key_id: String,
    pub status: MigrationManifestStatus,
}

impl MigrationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationManifestStatus) -> bool {
        match self.status {
            MigrationManifestStatus::Created => {
                match next {
                    MigrationManifestStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationManifestStatus::Validated => {
                match next {
                    MigrationManifestStatus::Published | MigrationManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationManifestStatus::Published => {
                return next == MigrationManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "migration manifest", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.margin - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationManifest {
        MigrationManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            starts_at: self.starts_at,
            margin: self.margin,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a migration manifest in status created.
pub fn new_migration_manifest(code: &str, label: &str) -> MigrationManifest {
    MigrationManifest {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        starts_at: 0,
        margin: 0.0,
        key_id: "".to_string(),
        status: MigrationManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_migration_manifest("code-268", "label-126");
        assert_eq!(item.status, MigrationManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_migration_manifest("code-268", "label-126");
        if item.move_to(MigrationManifestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationManifestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_migration_manifest("code-268", "label-126");
        item.status = MigrationManifestStatus::Retired;
        assert!(
            item.move_to(MigrationManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
