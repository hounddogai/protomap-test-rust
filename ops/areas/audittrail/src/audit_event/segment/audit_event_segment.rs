use super::*;

use acme_kit::text;

/// Where a audit event segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AuditEventSegmentStatus {
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

impl std::fmt::Display for AuditEventSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSegment {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub tenant_id: String,
    pub archived: bool,
    pub grace_days: i64,
    pub checksum: String,
    pub status: AuditEventSegmentStatus,
}

impl AuditEventSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventSegmentStatus) -> bool {
        match self.status {
            AuditEventSegmentStatus::Planned => {
                return next == AuditEventSegmentStatus::Scheduled || next == AuditEventSegmentStatus::Completed;
            }
            AuditEventSegmentStatus::Scheduled => {
                match next {
                    AuditEventSegmentStatus::InProgress | AuditEventSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventSegmentStatus::InProgress => {
                return next == AuditEventSegmentStatus::Cancelled;
            }
            AuditEventSegmentStatus::Completed => {
                return next == AuditEventSegmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.sequence as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventSegment {
        AuditEventSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            tenant_id: self.tenant_id.clone(),
            archived: self.archived,
            grace_days: self.grace_days,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a audit event segment in status planned.
pub fn new_audit_event_segment(code: &str, title: &str) -> AuditEventSegment {
    AuditEventSegment {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        tenant_id: "".to_string(),
        archived: false,
        grace_days: 0,
        checksum: "".to_string(),
        status: AuditEventSegmentStatus::Planned,
    }
}
