use super::*;

use acme_kit::text;

/// Where a manual migration manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualMigrationManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualMigrationManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualMigrationManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual migration manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualMigrationManifest {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub limit: i64,
    pub description: String,
    pub principal: String,
    pub segment: String,
    pub approved: bool,
    pub weight: f64,
    pub duration_minutes: i64,
    pub visible: bool,
    pub status: ManualMigrationManifestStatus,
}

impl ManualMigrationManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualMigrationManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualMigrationManifestStatus) -> bool {
        match self.status {
            ManualMigrationManifestStatus::Requested => {
                return next == ManualMigrationManifestStatus::Granted
                    || next == ManualMigrationManifestStatus::Expired;
            }
            ManualMigrationManifestStatus::Granted => {
                match next {
                    ManualMigrationManifestStatus::Revoked | ManualMigrationManifestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualMigrationManifestStatus::Revoked => {
                return next == ManualMigrationManifestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualMigrationManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual migration manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.weight + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualMigrationManifest {
        ManualMigrationManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            limit: self.limit,
            description: self.description.clone(),
            principal: self.principal.clone(),
            segment: self.segment.clone(),
            approved: self.approved,
            weight: self.weight,
            duration_minutes: self.duration_minutes,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a manual migration manifest in status requested.
pub fn new_manual_migration_manifest(code: &str, title: &str) -> ManualMigrationManifest {
    ManualMigrationManifest {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        limit: 0,
        description: "".to_string(),
        principal: "".to_string(),
        segment: "".to_string(),
        approved: false,
        weight: 0.0,
        duration_minutes: 0,
        visible: false,
        status: ManualMigrationManifestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_migration_manifest("code-29", "title-635");
        assert_eq!(item.status, ManualMigrationManifestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_migration_manifest("code-29", "title-635");
        if item.move_to(ManualMigrationManifestStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualMigrationManifestStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_migration_manifest("code-29", "title-635");
        item.status = ManualMigrationManifestStatus::Expired;
        assert!(
            item.move_to(ManualMigrationManifestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
