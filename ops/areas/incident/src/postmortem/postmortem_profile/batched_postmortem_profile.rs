use super::*;

use acme_kit::text;

/// Where a batched postmortem profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPostmortemProfileStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedPostmortemProfileStatus {
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

impl std::fmt::Display for BatchedPostmortemProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched postmortem profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemProfile {
    pub code: String,
    pub title: String,
    pub host: String,
    pub discount: f64,
    pub notes: String,
    pub audience: String,
    pub channel: String,
    pub status: BatchedPostmortemProfileStatus,
}

impl BatchedPostmortemProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPostmortemProfileStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPostmortemProfileStatus) -> bool {
        match self.status {
            BatchedPostmortemProfileStatus::New => {
                return next == BatchedPostmortemProfileStatus::Queued
                    || next == BatchedPostmortemProfileStatus::Running;
            }
            BatchedPostmortemProfileStatus::Queued => {
                return next == BatchedPostmortemProfileStatus::Done;
            }
            BatchedPostmortemProfileStatus::Running => {
                match next {
                    BatchedPostmortemProfileStatus::Done | BatchedPostmortemProfileStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPostmortemProfileStatus::Done => {
                return next == BatchedPostmortemProfileStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedPostmortemProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched postmortem profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> BatchedPostmortemProfile {
        BatchedPostmortemProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            discount: self.discount,
            notes: self.notes.clone(),
            audience: self.audience.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a batched postmortem profile in status new.
pub fn new_batched_postmortem_profile(code: &str, title: &str) -> BatchedPostmortemProfile {
    BatchedPostmortemProfile {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        discount: 0.0,
        notes: "".to_string(),
        audience: "".to_string(),
        channel: "".to_string(),
        status: BatchedPostmortemProfileStatus::New,
    }
}
