use super::*;

use acme_kit::text;

/// Where a rolling migration profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingMigrationProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingMigrationProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RollingMigrationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling migration profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingMigrationProfile {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub created_at: i64,
    pub floor_value: f64,
    pub principal: String,
    pub quota_units: i64,
    pub source: String,
    pub ceiling: f64,
    pub key_id: String,
    pub owner_id: String,
    pub status: RollingMigrationProfileStatus,
}

impl RollingMigrationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingMigrationProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingMigrationProfileStatus) -> bool {
        match self.status {
            RollingMigrationProfileStatus::Open => {
                return next == RollingMigrationProfileStatus::Resolved
                    || next == RollingMigrationProfileStatus::Reopened;
            }
            RollingMigrationProfileStatus::InReview => {
                return next == RollingMigrationProfileStatus::Archived;
            }
            RollingMigrationProfileStatus::Resolved => {
                return next == RollingMigrationProfileStatus::Reopened
                    || next == RollingMigrationProfileStatus::Archived;
            }
            RollingMigrationProfileStatus::Reopened => {
                return next == RollingMigrationProfileStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingMigrationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling migration profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.ceiling * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingMigrationProfile {
        RollingMigrationProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            created_at: self.created_at,
            floor_value: self.floor_value,
            principal: self.principal.clone(),
            quota_units: self.quota_units,
            source: self.source.clone(),
            ceiling: self.ceiling,
            key_id: self.key_id.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling migration profile in status open.
pub fn new_rolling_migration_profile(code: &str, name: &str) -> RollingMigrationProfile {
    RollingMigrationProfile {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        created_at: 0,
        floor_value: 0.0,
        principal: "".to_string(),
        quota_units: 0,
        source: "".to_string(),
        ceiling: 0.0,
        key_id: "".to_string(),
        owner_id: "".to_string(),
        status: RollingMigrationProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_migration_profile("code-210", "name-199");
        assert_eq!(item.status, RollingMigrationProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rolling_migration_profile("code-210", "name-199");
        if item.move_to(RollingMigrationProfileStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingMigrationProfileStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_migration_profile("code-210", "name-199");
        item.status = RollingMigrationProfileStatus::Archived;
        assert!(
            item.move_to(RollingMigrationProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
