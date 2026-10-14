use super::*;

use acme_kit::text;

/// Where a allowlist segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AllowlistSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AllowlistSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist segment as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSegment {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub threshold: f64,
    pub attempts: i64,
    pub burst: i64,
    pub ttl_seconds: i64,
    pub weight: f64,
    pub external_ref: String,
    pub contact_email: String,
    pub ceiling: f64,
    pub status: AllowlistSegmentStatus,
}

impl AllowlistSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistSegmentStatus) -> bool {
        match self.status {
            AllowlistSegmentStatus::Created => {
                return next == AllowlistSegmentStatus::Retired;
            }
            AllowlistSegmentStatus::Validated => {
                return next == AllowlistSegmentStatus::Published || next == AllowlistSegmentStatus::Retired;
            }
            AllowlistSegmentStatus::Published => {
                return next == AllowlistSegmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "allowlist segment", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.weight + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistSegment {
        AllowlistSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            threshold: self.threshold,
            attempts: self.attempts,
            burst: self.burst,
            ttl_seconds: self.ttl_seconds,
            weight: self.weight,
            external_ref: self.external_ref.clone(),
            contact_email: self.contact_email.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a allowlist segment in status created.
pub fn new_allowlist_segment(code: &str, label: &str) -> AllowlistSegment {
    AllowlistSegment {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        threshold: 0.0,
        attempts: 0,
        burst: 0,
        ttl_seconds: 0,
        weight: 0.0,
        external_ref: "".to_string(),
        contact_email: "".to_string(),
        ceiling: 0.0,
        status: AllowlistSegmentStatus::Created,
    }
}
