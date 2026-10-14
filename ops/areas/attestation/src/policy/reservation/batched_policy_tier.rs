use super::*;

use acme_kit::text;

/// Where a batched policy tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPolicyTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedPolicyTierStatus {
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

impl std::fmt::Display for BatchedPolicyTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched policy tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTier {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub title: String,
    pub duration_minutes: i64,
    pub status: BatchedPolicyTierStatus,
}

impl BatchedPolicyTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPolicyTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPolicyTierStatus) -> bool {
        match self.status {
            BatchedPolicyTierStatus::Draft => {
                return next == BatchedPolicyTierStatus::Active;
            }
            BatchedPolicyTierStatus::Pending => {
                match next {
                    BatchedPolicyTierStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyTierStatus::Active => {
                return next == BatchedPolicyTierStatus::Suspended || next == BatchedPolicyTierStatus::Closed;
            }
            BatchedPolicyTierStatus::Suspended => {
                match next {
                    BatchedPolicyTierStatus::Closed => {
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

    pub fn move_to(&mut self, next: BatchedPolicyTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched policy tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> BatchedPolicyTier {
        BatchedPolicyTier {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a batched policy tier in status draft.
pub fn new_batched_policy_tier(code: &str, name: &str) -> BatchedPolicyTier {
    BatchedPolicyTier {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        title: "".to_string(),
        duration_minutes: 0,
        status: BatchedPolicyTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_policy_tier("code-788", "name-939");
        assert_eq!(item.status, BatchedPolicyTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_batched_policy_tier("code-788", "name-939");
        if item.move_to(BatchedPolicyTierStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPolicyTierStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_policy_tier("code-788", "name-939");
        item.status = BatchedPolicyTierStatus::Closed;
        assert!(
            item.move_to(BatchedPolicyTierStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
