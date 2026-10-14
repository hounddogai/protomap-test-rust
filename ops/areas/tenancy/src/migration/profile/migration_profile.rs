use super::*;

use acme_kit::text;

/// Where a migration profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl MigrationProfileStatus {
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

impl std::fmt::Display for MigrationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration profile as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationProfile {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub external_ref: String,
    pub starts_at: i64,
    pub cost: f64,
    pub threshold: f64,
    pub status: MigrationProfileStatus,
}

impl MigrationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationProfileStatus) -> bool {
        match self.status {
            MigrationProfileStatus::Open => {
                return next == MigrationProfileStatus::Reopened || next == MigrationProfileStatus::Archived;
            }
            MigrationProfileStatus::InReview => {
                return next == MigrationProfileStatus::Reopened || next == MigrationProfileStatus::Archived;
            }
            MigrationProfileStatus::Resolved => {
                return next == MigrationProfileStatus::Reopened;
            }
            MigrationProfileStatus::Reopened => {
                match next {
                    MigrationProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration profile {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.cost * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MigrationProfile {
        MigrationProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            external_ref: self.external_ref.clone(),
            starts_at: self.starts_at,
            cost: self.cost,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a migration profile in status open.
pub fn new_migration_profile(code: &str, label: &str) -> MigrationProfile {
    MigrationProfile {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        external_ref: "".to_string(),
        starts_at: 0,
        cost: 0.0,
        threshold: 0.0,
        status: MigrationProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_migration_profile("code-250", "label-387");
        assert_eq!(item.status, MigrationProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_migration_profile("code-250", "label-387");
        if item.move_to(MigrationProfileStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationProfileStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_migration_profile("code-250", "label-387");
        item.status = MigrationProfileStatus::Archived;
        assert!(
            item.move_to(MigrationProfileStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
