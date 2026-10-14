use super::*;

use acme_kit::text;

/// Where a rolling ttl policy estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTtlPolicyEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingTtlPolicyEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollingTtlPolicyEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling ttl policy estimate as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyEstimate {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub priority: i64,
    pub active: bool,
    pub namespace: String,
    pub version: i64,
    pub status: RollingTtlPolicyEstimateStatus,
}

impl RollingTtlPolicyEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTtlPolicyEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTtlPolicyEstimateStatus) -> bool {
        match self.status {
            RollingTtlPolicyEstimateStatus::New => {
                match next {
                    RollingTtlPolicyEstimateStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTtlPolicyEstimateStatus::Queued => {
                return next == RollingTtlPolicyEstimateStatus::Done || next == RollingTtlPolicyEstimateStatus::Failed;
            }
            RollingTtlPolicyEstimateStatus::Running => {
                match next {
                    RollingTtlPolicyEstimateStatus::Done | RollingTtlPolicyEstimateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTtlPolicyEstimateStatus::Done => {
                return next == RollingTtlPolicyEstimateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingTtlPolicyEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling ttl policy estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling ttl policy estimate", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.priority as f64) * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTtlPolicyEstimate {
        RollingTtlPolicyEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            priority: self.priority,
            active: self.active,
            namespace: self.namespace.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a rolling ttl policy estimate in status new.
pub fn new_rolling_ttl_policy_estimate(code: &str, label: &str) -> RollingTtlPolicyEstimate {
    RollingTtlPolicyEstimate {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        priority: 0,
        active: false,
        namespace: "".to_string(),
        version: 0,
        status: RollingTtlPolicyEstimateStatus::New,
    }
}
