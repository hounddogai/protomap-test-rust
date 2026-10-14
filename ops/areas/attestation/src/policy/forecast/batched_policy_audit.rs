use super::*;

use acme_kit::text;

/// Where a batched policy audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPolicyAuditStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedPolicyAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedPolicyAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched policy audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAudit {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub notes: String,
    pub contact_email: String,
    pub title: String,
    pub amount: f64,
    pub status: BatchedPolicyAuditStatus,
}

impl BatchedPolicyAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPolicyAuditStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPolicyAuditStatus) -> bool {
        match self.status {
            BatchedPolicyAuditStatus::Proposed => {
                match next {
                    BatchedPolicyAuditStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyAuditStatus::Approved => {
                match next {
                    BatchedPolicyAuditStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyAuditStatus::Rejected => {
                return next == BatchedPolicyAuditStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPolicyAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched policy audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> BatchedPolicyAudit {
        BatchedPolicyAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            notes: self.notes.clone(),
            contact_email: self.contact_email.clone(),
            title: self.title.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched policy audit in status proposed.
pub fn new_batched_policy_audit(code: &str, label: &str) -> BatchedPolicyAudit {
    BatchedPolicyAudit {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        notes: "".to_string(),
        contact_email: "".to_string(),
        title: "".to_string(),
        amount: 0.0,
        status: BatchedPolicyAuditStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_policy_audit("code-313", "label-634");
        assert_eq!(item.status, BatchedPolicyAuditStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_batched_policy_audit("code-313", "label-634");
        if item.move_to(BatchedPolicyAuditStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPolicyAuditStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_policy_audit("code-313", "label-634");
        item.status = BatchedPolicyAuditStatus::Withdrawn;
        assert!(
            item.move_to(BatchedPolicyAuditStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
