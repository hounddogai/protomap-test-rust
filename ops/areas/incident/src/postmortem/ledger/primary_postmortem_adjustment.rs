use super::*;

use acme_kit::text;

/// Where a primary postmortem adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPostmortemAdjustmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryPostmortemAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PrimaryPostmortemAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary postmortem adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPostmortemAdjustment {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub capacity: i64,
    pub fingerprint: String,
    pub notes: String,
    pub status: PrimaryPostmortemAdjustmentStatus,
}

impl PrimaryPostmortemAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPostmortemAdjustmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPostmortemAdjustmentStatus) -> bool {
        match self.status {
            PrimaryPostmortemAdjustmentStatus::Open => {
                return next == PrimaryPostmortemAdjustmentStatus::Reopened;
            }
            PrimaryPostmortemAdjustmentStatus::InReview => {
                return next == PrimaryPostmortemAdjustmentStatus::Archived;
            }
            PrimaryPostmortemAdjustmentStatus::Resolved => {
                return next == PrimaryPostmortemAdjustmentStatus::Archived;
            }
            PrimaryPostmortemAdjustmentStatus::Reopened => {
                return next == PrimaryPostmortemAdjustmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryPostmortemAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary postmortem adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary postmortem adjustment", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.capacity as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPostmortemAdjustment {
        PrimaryPostmortemAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            capacity: self.capacity,
            fingerprint: self.fingerprint.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary postmortem adjustment in status open.
pub fn new_primary_postmortem_adjustment(code: &str, title: &str) -> PrimaryPostmortemAdjustment {
    PrimaryPostmortemAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        capacity: 0,
        fingerprint: "".to_string(),
        notes: "".to_string(),
        status: PrimaryPostmortemAdjustmentStatus::Open,
    }
}
