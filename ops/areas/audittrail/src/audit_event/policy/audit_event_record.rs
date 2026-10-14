use super::*;

use acme_kit::text;

/// Where a audit event record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventRecordStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AuditEventRecordStatus {
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

impl std::fmt::Display for AuditEventRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventRecord {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub rotation_days: i64,
    pub window_days: i64,
    pub floor_value: f64,
    pub tags: Vec<String>,
    pub timezone: String,
    pub burst: i64,
    pub status: AuditEventRecordStatus,
}

impl AuditEventRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventRecordStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventRecordStatus) -> bool {
        match self.status {
            AuditEventRecordStatus::Planned => {
                return next == AuditEventRecordStatus::InProgress;
            }
            AuditEventRecordStatus::Scheduled => {
                match next {
                    AuditEventRecordStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventRecordStatus::InProgress => {
                match next {
                    AuditEventRecordStatus::Completed | AuditEventRecordStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventRecordStatus::Completed => {
                return next == AuditEventRecordStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.burst as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventRecord {
        AuditEventRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            rotation_days: self.rotation_days,
            window_days: self.window_days,
            floor_value: self.floor_value,
            tags: self.tags.clone(),
            timezone: self.timezone.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a audit event record in status planned.
pub fn new_audit_event_record(code: &str, title: &str) -> AuditEventRecord {
    AuditEventRecord {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        rotation_days: 0,
        window_days: 0,
        floor_value: 0.0,
        tags: Vec::new(),
        timezone: "".to_string(),
        burst: 0,
        status: AuditEventRecordStatus::Planned,
    }
}
