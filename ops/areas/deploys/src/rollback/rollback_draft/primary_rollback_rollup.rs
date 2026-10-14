use super::*;

use acme_kit::text;

/// Where a primary rollback rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRollbackRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryRollbackRollupStatus {
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

impl std::fmt::Display for PrimaryRollbackRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary rollback rollup as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRollbackRollup {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub category: String,
    pub archived: bool,
    pub currency: String,
    pub source: String,
    pub quantity: i64,
    pub status: PrimaryRollbackRollupStatus,
}

impl PrimaryRollbackRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRollbackRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRollbackRollupStatus) -> bool {
        match self.status {
            PrimaryRollbackRollupStatus::Open => {
                match next {
                    PrimaryRollbackRollupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRollbackRollupStatus::InReview => {
                return next == PrimaryRollbackRollupStatus::Resolved;
            }
            PrimaryRollbackRollupStatus::Resolved => {
                match next {
                    PrimaryRollbackRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRollbackRollupStatus::Reopened => {
                return next == PrimaryRollbackRollupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryRollbackRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rollback rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> PrimaryRollbackRollup {
        PrimaryRollbackRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            category: self.category.clone(),
            archived: self.archived,
            currency: self.currency.clone(),
            source: self.source.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a primary rollback rollup in status open.
pub fn new_primary_rollback_rollup(code: &str, label: &str) -> PrimaryRollbackRollup {
    PrimaryRollbackRollup {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        category: "".to_string(),
        archived: false,
        currency: "".to_string(),
        source: "".to_string(),
        quantity: 0,
        status: PrimaryRollbackRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_rollback_rollup("code-681", "label-963");
        assert_eq!(item.status, PrimaryRollbackRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_primary_rollback_rollup("code-681", "label-963");
        if item.move_to(PrimaryRollbackRollupStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRollbackRollupStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_rollback_rollup("code-681", "label-963");
        item.status = PrimaryRollbackRollupStatus::Archived;
        assert!(
            item.move_to(PrimaryRollbackRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
