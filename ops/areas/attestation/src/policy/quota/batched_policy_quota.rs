use super::*;

use acme_kit::text;

/// Where a batched policy quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPolicyQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedPolicyQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedPolicyQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched policy quota as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyQuota {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub audience: String,
    pub window_days: i64,
    pub status: BatchedPolicyQuotaStatus,
}

impl BatchedPolicyQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPolicyQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPolicyQuotaStatus) -> bool {
        match self.status {
            BatchedPolicyQuotaStatus::Proposed => {
                return next == BatchedPolicyQuotaStatus::Approved || next == BatchedPolicyQuotaStatus::Rejected;
            }
            BatchedPolicyQuotaStatus::Approved => {
                match next {
                    BatchedPolicyQuotaStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicyQuotaStatus::Rejected => {
                return next == BatchedPolicyQuotaStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPolicyQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched policy quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched policy quota", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> BatchedPolicyQuota {
        BatchedPolicyQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            audience: self.audience.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a batched policy quota in status proposed.
pub fn new_batched_policy_quota(code: &str, label: &str) -> BatchedPolicyQuota {
    BatchedPolicyQuota {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        audience: "".to_string(),
        window_days: 0,
        status: BatchedPolicyQuotaStatus::Proposed,
    }
}
