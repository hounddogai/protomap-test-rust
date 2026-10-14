use super::*;

use acme_kit::text;

/// Where a silence segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SilenceSegmentStatus {
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

impl std::fmt::Display for SilenceSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSegment {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub host: String,
    pub discount: f64,
    pub amount: f64,
    pub audience: String,
    pub external_ref: String,
    pub fingerprint: String,
    pub status: SilenceSegmentStatus,
}

impl SilenceSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceSegmentStatus) -> bool {
        match self.status {
            SilenceSegmentStatus::New => {
                return next == SilenceSegmentStatus::Queued || next == SilenceSegmentStatus::Running;
            }
            SilenceSegmentStatus::Queued => {
                return next == SilenceSegmentStatus::Done;
            }
            SilenceSegmentStatus::Running => {
                return next == SilenceSegmentStatus::Done;
            }
            SilenceSegmentStatus::Done => {
                return next == SilenceSegmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.discount - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceSegment {
        SilenceSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            host: self.host.clone(),
            discount: self.discount,
            amount: self.amount,
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a silence segment in status new.
pub fn new_silence_segment(code: &str, label: &str) -> SilenceSegment {
    SilenceSegment {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        host: "".to_string(),
        discount: 0.0,
        amount: 0.0,
        audience: "".to_string(),
        external_ref: "".to_string(),
        fingerprint: "".to_string(),
        status: SilenceSegmentStatus::New,
    }
}
