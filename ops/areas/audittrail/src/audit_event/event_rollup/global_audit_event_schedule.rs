use super::*;

use acme_kit::text;

/// Where a global audit event schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalAuditEventScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalAuditEventScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for GlobalAuditEventScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global audit event schedule as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAuditEventSchedule {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub ratio: f64,
    pub archived: bool,
    pub amount: f64,
    pub issuer: String,
    pub tags: Vec<String>,
    pub minimum: f64,
    pub burst: i64,
    pub status: GlobalAuditEventScheduleStatus,
}

impl GlobalAuditEventSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalAuditEventScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalAuditEventScheduleStatus) -> bool {
        match self.status {
            GlobalAuditEventScheduleStatus::Draft => {
                match next {
                    GlobalAuditEventScheduleStatus::Pending | GlobalAuditEventScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAuditEventScheduleStatus::Pending => {
                match next {
                    GlobalAuditEventScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalAuditEventScheduleStatus::Active => {
                return next == GlobalAuditEventScheduleStatus::Suspended
                    || next == GlobalAuditEventScheduleStatus::Closed;
            }
            GlobalAuditEventScheduleStatus::Suspended => {
                match next {
                    GlobalAuditEventScheduleStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalAuditEventScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global audit event schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
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

    pub fn net_value(&self) -> f64 {
        ((self.starts_at as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalAuditEventSchedule {
        GlobalAuditEventSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            ratio: self.ratio,
            archived: self.archived,
            amount: self.amount,
            issuer: self.issuer.clone(),
            tags: self.tags.clone(),
            minimum: self.minimum,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a global audit event schedule in status draft.
pub fn new_global_audit_event_schedule(code: &str, title: &str) -> GlobalAuditEventSchedule {
    GlobalAuditEventSchedule {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        ratio: 0.0,
        archived: false,
        amount: 0.0,
        issuer: "".to_string(),
        tags: Vec::new(),
        minimum: 0.0,
        burst: 0,
        status: GlobalAuditEventScheduleStatus::Draft,
    }
}
