use super::*;

use acme_kit::text;

/// Where a pending audit event slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingAuditEventSlotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingAuditEventSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for PendingAuditEventSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending audit event slot as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventSlot {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub name: String,
    pub batch_size: i64,
    pub issuer: String,
    pub archived: bool,
    pub status: PendingAuditEventSlotStatus,
}

impl PendingAuditEventSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingAuditEventSlotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingAuditEventSlotStatus) -> bool {
        match self.status {
            PendingAuditEventSlotStatus::Idle => {
                return next == PendingAuditEventSlotStatus::Draining;
            }
            PendingAuditEventSlotStatus::Warming => {
                match next {
                    PendingAuditEventSlotStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingAuditEventSlotStatus::Ready => {
                return next == PendingAuditEventSlotStatus::Draining || next == PendingAuditEventSlotStatus::Stopped;
            }
            PendingAuditEventSlotStatus::Draining => {
                return next == PendingAuditEventSlotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingAuditEventSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending audit event slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending audit event slot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> PendingAuditEventSlot {
        PendingAuditEventSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            issuer: self.issuer.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending audit event slot in status idle.
pub fn new_pending_audit_event_slot(code: &str, title: &str) -> PendingAuditEventSlot {
    PendingAuditEventSlot {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        name: "".to_string(),
        batch_size: 0,
        issuer: "".to_string(),
        archived: false,
        status: PendingAuditEventSlotStatus::Idle,
    }
}
