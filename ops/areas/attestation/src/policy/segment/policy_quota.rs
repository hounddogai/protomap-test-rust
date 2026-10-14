use super::*;

use acme_kit::text;

/// Where a policy quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PolicyQuotaStatus {
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

impl std::fmt::Display for PolicyQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy quota as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyQuota {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub score: f64,
    pub source: String,
    pub audience: String,
    pub status: PolicyQuotaStatus,
}

impl PolicyQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyQuotaStatus) -> bool {
        match self.status {
            PolicyQuotaStatus::New => {
                match next {
                    PolicyQuotaStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyQuotaStatus::Queued => {
                return next == PolicyQuotaStatus::Running || next == PolicyQuotaStatus::Failed;
            }
            PolicyQuotaStatus::Running => {
                return next == PolicyQuotaStatus::Done || next == PolicyQuotaStatus::Failed;
            }
            PolicyQuotaStatus::Done => {
                return next == PolicyQuotaStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "policy quota", self.code)
    }

    pub fn net_load(&self) -> f64 {
        (self.score * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyQuota {
        PolicyQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            score: self.score,
            source: self.source.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a policy quota in status new.
pub fn new_policy_quota(code: &str, title: &str) -> PolicyQuota {
    PolicyQuota {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        score: 0.0,
        source: "".to_string(),
        audience: "".to_string(),
        status: PolicyQuotaStatus::New,
    }
}
