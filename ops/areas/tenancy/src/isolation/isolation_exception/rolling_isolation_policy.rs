use super::*;

use acme_kit::text;

/// Where a rolling isolation policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingIsolationPolicyStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingIsolationPolicyStatus {
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

impl std::fmt::Display for RollingIsolationPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling isolation policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationPolicy {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub notes: String,
    pub source: String,
    pub status: RollingIsolationPolicyStatus,
}

impl RollingIsolationPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingIsolationPolicyStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingIsolationPolicyStatus) -> bool {
        match self.status {
            RollingIsolationPolicyStatus::New => {
                match next {
                    RollingIsolationPolicyStatus::Queued | RollingIsolationPolicyStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingIsolationPolicyStatus::Queued => {
                return next == RollingIsolationPolicyStatus::Running;
            }
            RollingIsolationPolicyStatus::Running => {
                return next == RollingIsolationPolicyStatus::Failed;
            }
            RollingIsolationPolicyStatus::Done => {
                match next {
                    RollingIsolationPolicyStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingIsolationPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling isolation policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> RollingIsolationPolicy {
        RollingIsolationPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            notes: self.notes.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling isolation policy in status new.
pub fn new_rolling_isolation_policy(code: &str, title: &str) -> RollingIsolationPolicy {
    RollingIsolationPolicy {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        notes: "".to_string(),
        source: "".to_string(),
        status: RollingIsolationPolicyStatus::New,
    }
}
