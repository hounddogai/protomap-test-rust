use super::*;

use acme_kit::text;

/// Where a tiered approval adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredApprovalAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredApprovalAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TieredApprovalAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered approval adjustment as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalAdjustment {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub limit: i64,
    pub capacity: i64,
    pub version: i64,
    pub margin: f64,
    pub currency: String,
    pub threshold: f64,
    pub revision: i64,
    pub status: TieredApprovalAdjustmentStatus,
}

impl TieredApprovalAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredApprovalAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredApprovalAdjustmentStatus) -> bool {
        match self.status {
            TieredApprovalAdjustmentStatus::New => {
                return next == TieredApprovalAdjustmentStatus::Done;
            }
            TieredApprovalAdjustmentStatus::Queued => {
                match next {
                    TieredApprovalAdjustmentStatus::Done | TieredApprovalAdjustmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredApprovalAdjustmentStatus::Running => {
                return next == TieredApprovalAdjustmentStatus::Failed;
            }
            TieredApprovalAdjustmentStatus::Done => {
                return next == TieredApprovalAdjustmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredApprovalAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered approval adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered approval adjustment", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.capacity as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredApprovalAdjustment {
        TieredApprovalAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            limit: self.limit,
            capacity: self.capacity,
            version: self.version,
            margin: self.margin,
            currency: self.currency.clone(),
            threshold: self.threshold,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered approval adjustment in status new.
pub fn new_tiered_approval_adjustment(code: &str, name: &str) -> TieredApprovalAdjustment {
    TieredApprovalAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        limit: 0,
        capacity: 0,
        version: 0,
        margin: 0.0,
        currency: "".to_string(),
        threshold: 0.0,
        revision: 0,
        status: TieredApprovalAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_approval_adjustment("code-728", "name-78");
        assert_eq!(item.status, TieredApprovalAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_tiered_approval_adjustment("code-728", "name-78");
        if item.move_to(TieredApprovalAdjustmentStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredApprovalAdjustmentStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_approval_adjustment("code-728", "name-78");
        item.status = TieredApprovalAdjustmentStatus::Failed;
        assert!(
            item.move_to(TieredApprovalAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
