use super::*;

use acme_kit::text;

/// Where a batched binding policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedBindingPolicyStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedBindingPolicyStatus {
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

impl std::fmt::Display for BatchedBindingPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched binding policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingPolicy {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub pinned: bool,
    pub retries: i64,
    pub audience: String,
    pub created_at: i64,
    pub status: BatchedBindingPolicyStatus,
}

impl BatchedBindingPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedBindingPolicyStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedBindingPolicyStatus) -> bool {
        match self.status {
            BatchedBindingPolicyStatus::Draft => {
                return next == BatchedBindingPolicyStatus::Pending || next == BatchedBindingPolicyStatus::Closed;
            }
            BatchedBindingPolicyStatus::Pending => {
                match next {
                    BatchedBindingPolicyStatus::Active | BatchedBindingPolicyStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedBindingPolicyStatus::Active => {
                return next == BatchedBindingPolicyStatus::Suspended || next == BatchedBindingPolicyStatus::Closed;
            }
            BatchedBindingPolicyStatus::Suspended => {
                match next {
                    BatchedBindingPolicyStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedBindingPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched binding policy {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.sequence as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedBindingPolicy {
        BatchedBindingPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            pinned: self.pinned,
            retries: self.retries,
            audience: self.audience.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched binding policy in status draft.
pub fn new_batched_binding_policy(code: &str, label: &str) -> BatchedBindingPolicy {
    BatchedBindingPolicy {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        pinned: false,
        retries: 0,
        audience: "".to_string(),
        created_at: 0,
        status: BatchedBindingPolicyStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_binding_policy("code-868", "label-864");
        assert_eq!(item.status, BatchedBindingPolicyStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_batched_binding_policy("code-868", "label-864");
        if item.move_to(BatchedBindingPolicyStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedBindingPolicyStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_binding_policy("code-868", "label-864");
        item.status = BatchedBindingPolicyStatus::Closed;
        assert!(
            item.move_to(BatchedBindingPolicyStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
