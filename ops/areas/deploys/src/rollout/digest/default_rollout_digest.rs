use super::*;

use acme_kit::text;

/// Where a default rollout digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRolloutDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultRolloutDigestStatus {
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

impl std::fmt::Display for DefaultRolloutDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default rollout digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutDigest {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub budget: f64,
    pub timezone: String,
    pub status: DefaultRolloutDigestStatus,
}

impl DefaultRolloutDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRolloutDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRolloutDigestStatus) -> bool {
        match self.status {
            DefaultRolloutDigestStatus::New => {
                return next == DefaultRolloutDigestStatus::Running || next == DefaultRolloutDigestStatus::Failed;
            }
            DefaultRolloutDigestStatus::Queued => {
                return next == DefaultRolloutDigestStatus::Done;
            }
            DefaultRolloutDigestStatus::Running => {
                return next == DefaultRolloutDigestStatus::Failed;
            }
            DefaultRolloutDigestStatus::Done => {
                match next {
                    DefaultRolloutDigestStatus::Failed => {
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

    pub fn move_to(&mut self, next: DefaultRolloutDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rollout digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.sequence as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRolloutDigest {
        DefaultRolloutDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            budget: self.budget,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default rollout digest in status new.
pub fn new_default_rollout_digest(code: &str, label: &str) -> DefaultRolloutDigest {
    DefaultRolloutDigest {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        budget: 0.0,
        timezone: "".to_string(),
        status: DefaultRolloutDigestStatus::New,
    }
}
