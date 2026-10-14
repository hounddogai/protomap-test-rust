use super::*;

use acme_kit::text;

/// Where a migration quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationQuotaStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl MigrationQuotaStatus {
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

impl std::fmt::Display for MigrationQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A migration quota as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationQuota {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub pinned: bool,
    pub cluster: String,
    pub summary: String,
    pub status: MigrationQuotaStatus,
}

impl MigrationQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationQuotaStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationQuotaStatus) -> bool {
        match self.status {
            MigrationQuotaStatus::Open => {
                match next {
                    MigrationQuotaStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationQuotaStatus::InReview => {
                return next == MigrationQuotaStatus::Resolved || next == MigrationQuotaStatus::Reopened;
            }
            MigrationQuotaStatus::Resolved => {
                match next {
                    MigrationQuotaStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MigrationQuotaStatus::Reopened => {
                return next == MigrationQuotaStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MigrationQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> MigrationQuota {
        MigrationQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            pinned: self.pinned,
            cluster: self.cluster.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a migration quota in status open.
pub fn new_migration_quota(code: &str, name: &str) -> MigrationQuota {
    MigrationQuota {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        pinned: false,
        cluster: "".to_string(),
        summary: "".to_string(),
        status: MigrationQuotaStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_migration_quota("code-392", "name-421");
        assert_eq!(item.status, MigrationQuotaStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_migration_quota("code-392", "name-421");
        if item.move_to(MigrationQuotaStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MigrationQuotaStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_migration_quota("code-392", "name-421");
        item.status = MigrationQuotaStatus::Archived;
        assert!(
            item.move_to(MigrationQuotaStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
