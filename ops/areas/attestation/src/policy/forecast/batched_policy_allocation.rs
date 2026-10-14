use super::*;

use acme_kit::text;

/// Where a batched policy allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPolicyAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedPolicyAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedPolicyAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched policy allocation as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAllocation {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub phone: String,
    pub fingerprint: String,
    pub revision: i64,
    pub reviewer: String,
    pub status: BatchedPolicyAllocationStatus,
}

impl BatchedPolicyAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPolicyAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPolicyAllocationStatus) -> bool {
        match self.status {
            BatchedPolicyAllocationStatus::Proposed => {
                return next == BatchedPolicyAllocationStatus::Approved
                    || next == BatchedPolicyAllocationStatus::Rejected;
            }
            BatchedPolicyAllocationStatus::Approved => {
                match next {
                    BatchedPolicyAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyAllocationStatus::Rejected => {
                return next == BatchedPolicyAllocationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPolicyAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched policy allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched policy allocation", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.revision as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPolicyAllocation {
        BatchedPolicyAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            phone: self.phone.clone(),
            fingerprint: self.fingerprint.clone(),
            revision: self.revision,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched policy allocation in status proposed.
pub fn new_batched_policy_allocation(code: &str, label: &str) -> BatchedPolicyAllocation {
    BatchedPolicyAllocation {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        phone: "".to_string(),
        fingerprint: "".to_string(),
        revision: 0,
        reviewer: "".to_string(),
        status: BatchedPolicyAllocationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_policy_allocation("code-41", "label-361");
        assert_eq!(item.status, BatchedPolicyAllocationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_batched_policy_allocation("code-41", "label-361");
        if item.move_to(BatchedPolicyAllocationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPolicyAllocationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_policy_allocation("code-41", "label-361");
        item.status = BatchedPolicyAllocationStatus::Withdrawn;
        assert!(
            item.move_to(BatchedPolicyAllocationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
