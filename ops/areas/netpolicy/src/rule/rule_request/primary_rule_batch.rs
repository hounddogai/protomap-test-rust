use super::*;

use acme_kit::text;

/// Where a primary rule batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRuleBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryRuleBatchStatus {
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

impl std::fmt::Display for PrimaryRuleBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary rule batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleBatch {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub checksum: String,
    pub tenant_id: String,
    pub grace_days: i64,
    pub ratio: f64,
    pub name: String,
    pub namespace: String,
    pub sequence: i64,
    pub status: PrimaryRuleBatchStatus,
}

impl PrimaryRuleBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRuleBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRuleBatchStatus) -> bool {
        match self.status {
            PrimaryRuleBatchStatus::Open => {
                match next {
                    PrimaryRuleBatchStatus::Resolved | PrimaryRuleBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRuleBatchStatus::InReview => {
                match next {
                    PrimaryRuleBatchStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRuleBatchStatus::Resolved => {
                match next {
                    PrimaryRuleBatchStatus::Reopened | PrimaryRuleBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRuleBatchStatus::Reopened => {
                match next {
                    PrimaryRuleBatchStatus::Archived => {
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

    pub fn move_to(&mut self, next: PrimaryRuleBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rule batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        (self.amount * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRuleBatch {
        PrimaryRuleBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            checksum: self.checksum.clone(),
            tenant_id: self.tenant_id.clone(),
            grace_days: self.grace_days,
            ratio: self.ratio,
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a primary rule batch in status open.
pub fn new_primary_rule_batch(code: &str, title: &str) -> PrimaryRuleBatch {
    PrimaryRuleBatch {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        checksum: "".to_string(),
        tenant_id: "".to_string(),
        grace_days: 0,
        ratio: 0.0,
        name: "".to_string(),
        namespace: "".to_string(),
        sequence: 0,
        status: PrimaryRuleBatchStatus::Open,
    }
}
