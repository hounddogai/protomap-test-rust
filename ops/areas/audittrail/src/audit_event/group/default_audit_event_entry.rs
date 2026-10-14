use super::*;

use acme_kit::text;

/// Where a default audit event entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultAuditEventEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultAuditEventEntryStatus {
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

impl std::fmt::Display for DefaultAuditEventEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default audit event entry as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventEntry {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub category: String,
    pub active: bool,
    pub title: String,
    pub status: DefaultAuditEventEntryStatus,
}

impl DefaultAuditEventEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultAuditEventEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultAuditEventEntryStatus) -> bool {
        match self.status {
            DefaultAuditEventEntryStatus::Planned => {
                match next {
                    DefaultAuditEventEntryStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultAuditEventEntryStatus::Scheduled => {
                match next {
                    DefaultAuditEventEntryStatus::Completed | DefaultAuditEventEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultAuditEventEntryStatus::InProgress => {
                match next {
                    DefaultAuditEventEntryStatus::Completed | DefaultAuditEventEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultAuditEventEntryStatus::Completed => {
                match next {
                    DefaultAuditEventEntryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: DefaultAuditEventEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default audit event entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> DefaultAuditEventEntry {
        DefaultAuditEventEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            category: self.category.clone(),
            active: self.active,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a default audit event entry in status planned.
pub fn new_default_audit_event_entry(code: &str, label: &str) -> DefaultAuditEventEntry {
    DefaultAuditEventEntry {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        category: "".to_string(),
        active: false,
        title: "".to_string(),
        status: DefaultAuditEventEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_audit_event_entry("code-377", "label-176");
        assert_eq!(item.status, DefaultAuditEventEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_default_audit_event_entry("code-377", "label-176");
        if item.move_to(DefaultAuditEventEntryStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultAuditEventEntryStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_audit_event_entry("code-377", "label-176");
        item.status = DefaultAuditEventEntryStatus::Cancelled;
        assert!(
            item.move_to(DefaultAuditEventEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
