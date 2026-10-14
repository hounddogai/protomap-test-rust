use super::*;

use acme_kit::text;

/// Where a batched policy baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPolicyBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedPolicyBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedPolicyBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched policy baseline as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyBaseline {
    pub code: String,
    pub label: String,
    pub category: String,
    pub locale: String,
    pub duration_minutes: i64,
    pub key_id: String,
    pub owner_id: String,
    pub timezone: String,
    pub principal: String,
    pub namespace: String,
    pub status: BatchedPolicyBaselineStatus,
}

impl BatchedPolicyBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPolicyBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPolicyBaselineStatus) -> bool {
        match self.status {
            BatchedPolicyBaselineStatus::Proposed => {
                match next {
                    BatchedPolicyBaselineStatus::Rejected | BatchedPolicyBaselineStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyBaselineStatus::Approved => {
                return next == BatchedPolicyBaselineStatus::Rejected || next == BatchedPolicyBaselineStatus::Withdrawn;
            }
            BatchedPolicyBaselineStatus::Rejected => {
                return next == BatchedPolicyBaselineStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPolicyBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched policy baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched policy baseline", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> BatchedPolicyBaseline {
        BatchedPolicyBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            locale: self.locale.clone(),
            duration_minutes: self.duration_minutes,
            key_id: self.key_id.clone(),
            owner_id: self.owner_id.clone(),
            timezone: self.timezone.clone(),
            principal: self.principal.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched policy baseline in status proposed.
pub fn new_batched_policy_baseline(code: &str, label: &str) -> BatchedPolicyBaseline {
    BatchedPolicyBaseline {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        locale: "".to_string(),
        duration_minutes: 0,
        key_id: "".to_string(),
        owner_id: "".to_string(),
        timezone: "".to_string(),
        principal: "".to_string(),
        namespace: "".to_string(),
        status: BatchedPolicyBaselineStatus::Proposed,
    }
}
