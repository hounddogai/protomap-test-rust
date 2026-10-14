use super::*;

use acme_kit::text;

/// Where a scheduled sink digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSinkDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledSinkDigestStatus {
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

impl std::fmt::Display for ScheduledSinkDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled sink digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkDigest {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub grace_days: i64,
    pub starts_at: i64,
    pub updated_at: i64,
    pub principal: String,
    pub status: ScheduledSinkDigestStatus,
}

impl ScheduledSinkDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSinkDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSinkDigestStatus) -> bool {
        match self.status {
            ScheduledSinkDigestStatus::New => {
                match next {
                    ScheduledSinkDigestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSinkDigestStatus::Queued => {
                return next == ScheduledSinkDigestStatus::Done;
            }
            ScheduledSinkDigestStatus::Running => {
                match next {
                    ScheduledSinkDigestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSinkDigestStatus::Done => {
                return next == ScheduledSinkDigestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledSinkDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled sink digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled sink digest", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.grace_days as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSinkDigest {
        ScheduledSinkDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            grace_days: self.grace_days,
            starts_at: self.starts_at,
            updated_at: self.updated_at,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled sink digest in status new.
pub fn new_scheduled_sink_digest(code: &str, label: &str) -> ScheduledSinkDigest {
    ScheduledSinkDigest {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        grace_days: 0,
        starts_at: 0,
        updated_at: 0,
        principal: "".to_string(),
        status: ScheduledSinkDigestStatus::New,
    }
}
