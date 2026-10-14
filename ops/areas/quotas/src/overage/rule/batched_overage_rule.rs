use super::*;

use acme_kit::text;

/// Where a batched overage rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedOverageRuleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedOverageRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedOverageRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched overage rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedOverageRule {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub tenant_id: String,
    pub port: i64,
    pub starts_at: i64,
    pub status: BatchedOverageRuleStatus,
}

impl BatchedOverageRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedOverageRuleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedOverageRuleStatus) -> bool {
        match self.status {
            BatchedOverageRuleStatus::Proposed => {
                return next == BatchedOverageRuleStatus::Approved || next == BatchedOverageRuleStatus::Rejected;
            }
            BatchedOverageRuleStatus::Approved => {
                return next == BatchedOverageRuleStatus::Withdrawn;
            }
            BatchedOverageRuleStatus::Rejected => {
                return next == BatchedOverageRuleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedOverageRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched overage rule {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.port as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedOverageRule {
        BatchedOverageRule {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            tenant_id: self.tenant_id.clone(),
            port: self.port,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched overage rule in status proposed.
pub fn new_batched_overage_rule(code: &str, name: &str) -> BatchedOverageRule {
    BatchedOverageRule {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        tenant_id: "".to_string(),
        port: 0,
        starts_at: 0,
        status: BatchedOverageRuleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_overage_rule("code-968", "name-956");
        assert_eq!(item.status, BatchedOverageRuleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_batched_overage_rule("code-968", "name-956");
        if item.move_to(BatchedOverageRuleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedOverageRuleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_overage_rule("code-968", "name-956");
        item.status = BatchedOverageRuleStatus::Withdrawn;
        assert!(
            item.move_to(BatchedOverageRuleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
