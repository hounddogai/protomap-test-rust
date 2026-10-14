use super::*;

use acme_kit::text;

/// Where a global ttl policy request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalTtlPolicyRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalTtlPolicyRequestStatus {
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

impl std::fmt::Display for GlobalTtlPolicyRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global ttl policy request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTtlPolicyRequest {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub description: String,
    pub active: bool,
    pub expires_at: i64,
    pub retries: i64,
    pub burst: i64,
    pub timezone: String,
    pub status: GlobalTtlPolicyRequestStatus,
}

impl GlobalTtlPolicyRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalTtlPolicyRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalTtlPolicyRequestStatus) -> bool {
        match self.status {
            GlobalTtlPolicyRequestStatus::New => {
                return next == GlobalTtlPolicyRequestStatus::Failed;
            }
            GlobalTtlPolicyRequestStatus::Queued => {
                match next {
                    GlobalTtlPolicyRequestStatus::Running | GlobalTtlPolicyRequestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTtlPolicyRequestStatus::Running => {
                match next {
                    GlobalTtlPolicyRequestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalTtlPolicyRequestStatus::Done => {
                match next {
                    GlobalTtlPolicyRequestStatus::Failed => {
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

    pub fn move_to(&mut self, next: GlobalTtlPolicyRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global ttl policy request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.expires_at as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalTtlPolicyRequest {
        GlobalTtlPolicyRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            description: self.description.clone(),
            active: self.active,
            expires_at: self.expires_at,
            retries: self.retries,
            burst: self.burst,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global ttl policy request in status new.
pub fn new_global_ttl_policy_request(code: &str, label: &str) -> GlobalTtlPolicyRequest {
    GlobalTtlPolicyRequest {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        description: "".to_string(),
        active: false,
        expires_at: 0,
        retries: 0,
        burst: 0,
        timezone: "".to_string(),
        status: GlobalTtlPolicyRequestStatus::New,
    }
}
