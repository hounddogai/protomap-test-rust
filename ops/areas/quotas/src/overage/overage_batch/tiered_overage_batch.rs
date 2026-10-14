use super::*;

use acme_kit::text;

/// Where a tiered overage batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredOverageBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredOverageBatchStatus {
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

impl std::fmt::Display for TieredOverageBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered overage batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageBatch {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub margin: f64,
    pub scope: String,
    pub unit: String,
    pub status: TieredOverageBatchStatus,
}

impl TieredOverageBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredOverageBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredOverageBatchStatus) -> bool {
        match self.status {
            TieredOverageBatchStatus::Open => {
                return next == TieredOverageBatchStatus::InReview || next == TieredOverageBatchStatus::Reopened;
            }
            TieredOverageBatchStatus::InReview => {
                return next == TieredOverageBatchStatus::Reopened;
            }
            TieredOverageBatchStatus::Resolved => {
                match next {
                    TieredOverageBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageBatchStatus::Reopened => {
                return next == TieredOverageBatchStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredOverageBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered overage batch {} cannot move from {} to {}",
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

    pub fn copy(&self) -> TieredOverageBatch {
        TieredOverageBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            margin: self.margin,
            scope: self.scope.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered overage batch in status open.
pub fn new_tiered_overage_batch(code: &str, name: &str) -> TieredOverageBatch {
    TieredOverageBatch {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        margin: 0.0,
        scope: "".to_string(),
        unit: "".to_string(),
        status: TieredOverageBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_overage_batch("code-482", "name-306");
        assert_eq!(item.status, TieredOverageBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_tiered_overage_batch("code-482", "name-306");
        if item.move_to(TieredOverageBatchStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredOverageBatchStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_overage_batch("code-482", "name-306");
        item.status = TieredOverageBatchStatus::Archived;
        assert!(
            item.move_to(TieredOverageBatchStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
