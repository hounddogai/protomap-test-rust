use super::*;

use acme_kit::text;

/// Where a manual audit event entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualAuditEventEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualAuditEventEntryStatus {
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

impl std::fmt::Display for ManualAuditEventEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual audit event entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventEntry {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub unit: String,
    pub owner_id: String,
    pub amount: f64,
    pub status: ManualAuditEventEntryStatus,
}

impl ManualAuditEventEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualAuditEventEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualAuditEventEntryStatus) -> bool {
        match self.status {
            ManualAuditEventEntryStatus::Idle => {
                match next {
                    ManualAuditEventEntryStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualAuditEventEntryStatus::Warming => {
                match next {
                    ManualAuditEventEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualAuditEventEntryStatus::Ready => {
                match next {
                    ManualAuditEventEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualAuditEventEntryStatus::Draining => {
                return next == ManualAuditEventEntryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualAuditEventEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual audit event entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> ManualAuditEventEntry {
        ManualAuditEventEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            unit: self.unit.clone(),
            owner_id: self.owner_id.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a manual audit event entry in status idle.
pub fn new_manual_audit_event_entry(code: &str, title: &str) -> ManualAuditEventEntry {
    ManualAuditEventEntry {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        unit: "".to_string(),
        owner_id: "".to_string(),
        amount: 0.0,
        status: ManualAuditEventEntryStatus::Idle,
    }
}
