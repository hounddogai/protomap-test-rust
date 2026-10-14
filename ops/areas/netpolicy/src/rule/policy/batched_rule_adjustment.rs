use super::*;

use acme_kit::text;

/// Where a batched rule adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRuleAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedRuleAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedRuleAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched rule adjustment as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAdjustment {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub priority: i64,
    pub segment: String,
    pub window_days: i64,
    pub currency: String,
    pub status: BatchedRuleAdjustmentStatus,
}

impl BatchedRuleAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRuleAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRuleAdjustmentStatus) -> bool {
        match self.status {
            BatchedRuleAdjustmentStatus::Created => {
                return next == BatchedRuleAdjustmentStatus::Validated;
            }
            BatchedRuleAdjustmentStatus::Validated => {
                match next {
                    BatchedRuleAdjustmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRuleAdjustmentStatus::Published => {
                match next {
                    BatchedRuleAdjustmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: BatchedRuleAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched rule adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched rule adjustment", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.priority as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRuleAdjustment {
        BatchedRuleAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            priority: self.priority,
            segment: self.segment.clone(),
            window_days: self.window_days,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a batched rule adjustment in status created.
pub fn new_batched_rule_adjustment(code: &str, title: &str) -> BatchedRuleAdjustment {
    BatchedRuleAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        priority: 0,
        segment: "".to_string(),
        window_days: 0,
        currency: "".to_string(),
        status: BatchedRuleAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_rule_adjustment("code-414", "title-886");
        assert_eq!(item.status, BatchedRuleAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_batched_rule_adjustment("code-414", "title-886");
        if item.move_to(BatchedRuleAdjustmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRuleAdjustmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_rule_adjustment("code-414", "title-886");
        item.status = BatchedRuleAdjustmentStatus::Retired;
        assert!(
            item.move_to(BatchedRuleAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
