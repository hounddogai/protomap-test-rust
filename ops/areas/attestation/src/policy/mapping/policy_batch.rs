use super::*;

use acme_kit::text;

/// Where a policy batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyBatchStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PolicyBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PolicyBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy batch as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyBatch {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub rate: f64,
    pub category: String,
    pub key_id: String,
    pub currency: String,
    pub priority: i64,
    pub grace_days: i64,
    pub ratio: f64,
    pub summary: String,
    pub status: PolicyBatchStatus,
}

impl PolicyBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyBatchStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyBatchStatus) -> bool {
        match self.status {
            PolicyBatchStatus::Proposed => {
                match next {
                    PolicyBatchStatus::Approved | PolicyBatchStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyBatchStatus::Approved => {
                match next {
                    PolicyBatchStatus::Rejected | PolicyBatchStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyBatchStatus::Rejected => {
                return next == PolicyBatchStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "policy batch", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.priority as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyBatch {
        PolicyBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            rate: self.rate,
            category: self.category.clone(),
            key_id: self.key_id.clone(),
            currency: self.currency.clone(),
            priority: self.priority,
            grace_days: self.grace_days,
            ratio: self.ratio,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a policy batch in status proposed.
pub fn new_policy_batch(code: &str, name: &str) -> PolicyBatch {
    PolicyBatch {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        rate: 0.0,
        category: "".to_string(),
        key_id: "".to_string(),
        currency: "".to_string(),
        priority: 0,
        grace_days: 0,
        ratio: 0.0,
        summary: "".to_string(),
        status: PolicyBatchStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_policy_batch("code-687", "name-34");
        assert_eq!(item.status, PolicyBatchStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_policy_batch("code-687", "name-34");
        if item.move_to(PolicyBatchStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyBatchStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_policy_batch("code-687", "name-34");
        item.status = PolicyBatchStatus::Withdrawn;
        assert!(
            item.move_to(PolicyBatchStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
