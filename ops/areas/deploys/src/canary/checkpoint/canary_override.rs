use super::*;

use acme_kit::text;

/// Where a canary override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CanaryOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for CanaryOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryOverride {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub score: f64,
    pub window_days: i64,
    pub cipher: String,
    pub ends_at: i64,
    pub status: CanaryOverrideStatus,
}

impl CanaryOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryOverrideStatus) -> bool {
        match self.status {
            CanaryOverrideStatus::Planned => {
                return next == CanaryOverrideStatus::Scheduled || next == CanaryOverrideStatus::InProgress;
            }
            CanaryOverrideStatus::Scheduled => {
                match next {
                    CanaryOverrideStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryOverrideStatus::InProgress => {
                return next == CanaryOverrideStatus::Completed || next == CanaryOverrideStatus::Cancelled;
            }
            CanaryOverrideStatus::Completed => {
                match next {
                    CanaryOverrideStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CanaryOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.score + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryOverride {
        CanaryOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            score: self.score,
            window_days: self.window_days,
            cipher: self.cipher.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a canary override in status planned.
pub fn new_canary_override(code: &str, title: &str) -> CanaryOverride {
    CanaryOverride {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        score: 0.0,
        window_days: 0,
        cipher: "".to_string(),
        ends_at: 0,
        status: CanaryOverrideStatus::Planned,
    }
}
