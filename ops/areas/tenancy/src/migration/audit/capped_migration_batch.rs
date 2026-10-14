use super::*;

use acme_kit::text;

/// Where a capped migration batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedMigrationBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedMigrationBatchStatus {
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

impl std::fmt::Display for CappedMigrationBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped migration batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationBatch {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub ratio: f64,
    pub namespace: String,
    pub label: String,
    pub cluster: String,
    pub status: CappedMigrationBatchStatus,
}

impl CappedMigrationBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedMigrationBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedMigrationBatchStatus) -> bool {
        match self.status {
            CappedMigrationBatchStatus::Open => {
                match next {
                    CappedMigrationBatchStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedMigrationBatchStatus::InReview => {
                match next {
                    CappedMigrationBatchStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedMigrationBatchStatus::Resolved => {
                return next == CappedMigrationBatchStatus::Reopened;
            }
            CappedMigrationBatchStatus::Reopened => {
                return next == CappedMigrationBatchStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedMigrationBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped migration batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.floor_value + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedMigrationBatch {
        CappedMigrationBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            ratio: self.ratio,
            namespace: self.namespace.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a capped migration batch in status open.
pub fn new_capped_migration_batch(code: &str, name: &str) -> CappedMigrationBatch {
    CappedMigrationBatch {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        ratio: 0.0,
        namespace: "".to_string(),
        label: "".to_string(),
        cluster: "".to_string(),
        status: CappedMigrationBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_migration_batch("code-319", "name-570");
        assert_eq!(item.status, CappedMigrationBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_capped_migration_batch("code-319", "name-570");
        if item.move_to(CappedMigrationBatchStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedMigrationBatchStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_migration_batch("code-319", "name-570");
        item.status = CappedMigrationBatchStatus::Archived;
        assert!(
            item.move_to(CappedMigrationBatchStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
