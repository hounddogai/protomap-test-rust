use super::*;

use acme_kit::text;

/// Where a pending ttl policy threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTtlPolicyThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PendingTtlPolicyThresholdStatus {
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

impl std::fmt::Display for PendingTtlPolicyThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending ttl policy threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyThreshold {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub sequence: i64,
    pub rotation_days: i64,
    pub status: PendingTtlPolicyThresholdStatus,
}

impl PendingTtlPolicyThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTtlPolicyThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTtlPolicyThresholdStatus) -> bool {
        match self.status {
            PendingTtlPolicyThresholdStatus::Draft => {
                match next {
                    PendingTtlPolicyThresholdStatus::Pending | PendingTtlPolicyThresholdStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTtlPolicyThresholdStatus::Pending => {
                match next {
                    PendingTtlPolicyThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTtlPolicyThresholdStatus::Active => {
                return next == PendingTtlPolicyThresholdStatus::Suspended;
            }
            PendingTtlPolicyThresholdStatus::Suspended => {
                return next == PendingTtlPolicyThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingTtlPolicyThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending ttl policy threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending ttl policy threshold", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.rotation_days as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTtlPolicyThreshold {
        PendingTtlPolicyThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            sequence: self.sequence,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending ttl policy threshold in status draft.
pub fn new_pending_ttl_policy_threshold(code: &str, title: &str) -> PendingTtlPolicyThreshold {
    PendingTtlPolicyThreshold {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        sequence: 0,
        rotation_days: 0,
        status: PendingTtlPolicyThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_pending_ttl_policy_threshold("code-51", "title-22");
        assert_eq!(item.status, PendingTtlPolicyThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_pending_ttl_policy_threshold("code-51", "title-22");
        if item.move_to(PendingTtlPolicyThresholdStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTtlPolicyThresholdStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_pending_ttl_policy_threshold("code-51", "title-22");
        item.status = PendingTtlPolicyThresholdStatus::Closed;
        assert!(
            item.move_to(PendingTtlPolicyThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
