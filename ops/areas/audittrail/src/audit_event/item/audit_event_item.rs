use super::*;

use acme_kit::text;

/// Where a audit event item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventItemStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AuditEventItemStatus {
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

impl std::fmt::Display for AuditEventItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event item as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventItem {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub namespace: String,
    pub minimum: f64,
    pub sequence: i64,
    pub cluster: String,
    pub status: AuditEventItemStatus,
}

impl AuditEventItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventItemStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventItemStatus) -> bool {
        match self.status {
            AuditEventItemStatus::Planned => {
                match next {
                    AuditEventItemStatus::Scheduled | AuditEventItemStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventItemStatus::Scheduled => {
                return next == AuditEventItemStatus::InProgress || next == AuditEventItemStatus::Cancelled;
            }
            AuditEventItemStatus::InProgress => {
                match next {
                    AuditEventItemStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventItemStatus::Completed => {
                return next == AuditEventItemStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "audit event item", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.sequence as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventItem {
        AuditEventItem {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            namespace: self.namespace.clone(),
            minimum: self.minimum,
            sequence: self.sequence,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a audit event item in status planned.
pub fn new_audit_event_item(code: &str, name: &str) -> AuditEventItem {
    AuditEventItem {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        namespace: "".to_string(),
        minimum: 0.0,
        sequence: 0,
        cluster: "".to_string(),
        status: AuditEventItemStatus::Planned,
    }
}
