use super::*;

use acme_kit::text;

/// Where a batched retention rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRetentionRuleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedRetentionRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BatchedRetentionRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched retention rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionRule {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub retries: i64,
    pub unit: String,
    pub rate: f64,
    pub ends_at: i64,
    pub weight: f64,
    pub discount: f64,
    pub amount: f64,
    pub budget: f64,
    pub status: BatchedRetentionRuleStatus,
}

impl BatchedRetentionRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRetentionRuleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRetentionRuleStatus) -> bool {
        match self.status {
            BatchedRetentionRuleStatus::Draft => {
                match next {
                    BatchedRetentionRuleStatus::Suspended | BatchedRetentionRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRetentionRuleStatus::Pending => {
                return next == BatchedRetentionRuleStatus::Closed;
            }
            BatchedRetentionRuleStatus::Active => {
                match next {
                    BatchedRetentionRuleStatus::Suspended | BatchedRetentionRuleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRetentionRuleStatus::Suspended => {
                match next {
                    BatchedRetentionRuleStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedRetentionRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched retention rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.weight * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRetentionRule {
        BatchedRetentionRule {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            retries: self.retries,
            unit: self.unit.clone(),
            rate: self.rate,
            ends_at: self.ends_at,
            weight: self.weight,
            discount: self.discount,
            amount: self.amount,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a batched retention rule in status draft.
pub fn new_batched_retention_rule(code: &str, name: &str) -> BatchedRetentionRule {
    BatchedRetentionRule {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        retries: 0,
        unit: "".to_string(),
        rate: 0.0,
        ends_at: 0,
        weight: 0.0,
        discount: 0.0,
        amount: 0.0,
        budget: 0.0,
        status: BatchedRetentionRuleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_retention_rule("code-738", "name-812");
        assert_eq!(item.status, BatchedRetentionRuleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_batched_retention_rule("code-738", "name-812");
        if item.move_to(BatchedRetentionRuleStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRetentionRuleStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_retention_rule("code-738", "name-812");
        item.status = BatchedRetentionRuleStatus::Closed;
        assert!(
            item.move_to(BatchedRetentionRuleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
