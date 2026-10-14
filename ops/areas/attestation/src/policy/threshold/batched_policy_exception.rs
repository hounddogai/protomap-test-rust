use super::*;

use acme_kit::text;

/// Where a batched policy exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPolicyExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedPolicyExceptionStatus {
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

impl std::fmt::Display for BatchedPolicyExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched policy exception as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyException {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub owner_id: String,
    pub title: String,
    pub version: i64,
    pub status: BatchedPolicyExceptionStatus,
}

impl BatchedPolicyException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPolicyExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPolicyExceptionStatus) -> bool {
        match self.status {
            BatchedPolicyExceptionStatus::Draft => {
                match next {
                    BatchedPolicyExceptionStatus::Pending | BatchedPolicyExceptionStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyExceptionStatus::Pending => {
                return next == BatchedPolicyExceptionStatus::Active || next == BatchedPolicyExceptionStatus::Closed;
            }
            BatchedPolicyExceptionStatus::Active => {
                match next {
                    BatchedPolicyExceptionStatus::Suspended | BatchedPolicyExceptionStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyExceptionStatus::Suspended => {
                match next {
                    BatchedPolicyExceptionStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedPolicyExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched policy exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched policy exception", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> BatchedPolicyException {
        BatchedPolicyException {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            owner_id: self.owner_id.clone(),
            title: self.title.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched policy exception in status draft.
pub fn new_batched_policy_exception(code: &str, label: &str) -> BatchedPolicyException {
    BatchedPolicyException {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        owner_id: "".to_string(),
        title: "".to_string(),
        version: 0,
        status: BatchedPolicyExceptionStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_policy_exception("code-329", "label-86");
        assert_eq!(item.status, BatchedPolicyExceptionStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_batched_policy_exception("code-329", "label-86");
        if item.move_to(BatchedPolicyExceptionStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPolicyExceptionStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_policy_exception("code-329", "label-86");
        item.status = BatchedPolicyExceptionStatus::Closed;
        assert!(
            item.move_to(BatchedPolicyExceptionStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
