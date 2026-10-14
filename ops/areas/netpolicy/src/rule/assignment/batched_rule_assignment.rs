use super::*;

use acme_kit::text;

/// Where a batched rule assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRuleAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedRuleAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedRuleAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched rule assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRuleAssignment {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub quantity: i64,
    pub reviewer: String,
    pub archived: bool,
    pub version: i64,
    pub quota_units: i64,
    pub threshold: f64,
    pub status: BatchedRuleAssignmentStatus,
}

impl BatchedRuleAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRuleAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRuleAssignmentStatus) -> bool {
        match self.status {
            BatchedRuleAssignmentStatus::Created => {
                return next == BatchedRuleAssignmentStatus::Published || next == BatchedRuleAssignmentStatus::Retired;
            }
            BatchedRuleAssignmentStatus::Validated => {
                match next {
                    BatchedRuleAssignmentStatus::Published | BatchedRuleAssignmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRuleAssignmentStatus::Published => {
                return next == BatchedRuleAssignmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedRuleAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched rule assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched rule assignment", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quota_units as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRuleAssignment {
        BatchedRuleAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            quantity: self.quantity,
            reviewer: self.reviewer.clone(),
            archived: self.archived,
            version: self.version,
            quota_units: self.quota_units,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched rule assignment in status created.
pub fn new_batched_rule_assignment(code: &str, label: &str) -> BatchedRuleAssignment {
    BatchedRuleAssignment {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        quantity: 0,
        reviewer: "".to_string(),
        archived: false,
        version: 0,
        quota_units: 0,
        threshold: 0.0,
        status: BatchedRuleAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_rule_assignment("code-106", "label-971");
        assert_eq!(item.status, BatchedRuleAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_batched_rule_assignment("code-106", "label-971");
        if item.move_to(BatchedRuleAssignmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRuleAssignmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_rule_assignment("code-106", "label-971");
        item.status = BatchedRuleAssignmentStatus::Retired;
        assert!(
            item.move_to(BatchedRuleAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
