use super::*;

use acme_kit::text;

/// Where a pending retention segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRetentionSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingRetentionSegmentStatus {
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

impl std::fmt::Display for PendingRetentionSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending retention segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionSegment {
    pub code: String,
    pub name: String,
    pub host: String,
    pub contact_email: String,
    pub approved: bool,
    pub cipher: String,
    pub margin: f64,
    pub status: PendingRetentionSegmentStatus,
}

impl PendingRetentionSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRetentionSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRetentionSegmentStatus) -> bool {
        match self.status {
            PendingRetentionSegmentStatus::New => {
                match next {
                    PendingRetentionSegmentStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRetentionSegmentStatus::Queued => {
                match next {
                    PendingRetentionSegmentStatus::Running | PendingRetentionSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRetentionSegmentStatus::Running => {
                return next == PendingRetentionSegmentStatus::Failed;
            }
            PendingRetentionSegmentStatus::Done => {
                return next == PendingRetentionSegmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRetentionSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending retention segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending retention segment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> PendingRetentionSegment {
        PendingRetentionSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            contact_email: self.contact_email.clone(),
            approved: self.approved,
            cipher: self.cipher.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending retention segment in status new.
pub fn new_pending_retention_segment(code: &str, name: &str) -> PendingRetentionSegment {
    PendingRetentionSegment {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        contact_email: "".to_string(),
        approved: false,
        cipher: "".to_string(),
        margin: 0.0,
        status: PendingRetentionSegmentStatus::New,
    }
}
