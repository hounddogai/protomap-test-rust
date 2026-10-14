use super::*;

use acme_kit::text;

/// Where a audit event entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AuditEventEntryStatus {
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

impl std::fmt::Display for AuditEventEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event entry as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventEntry {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub cost: f64,
    pub priority: i64,
    pub namespace: String,
    pub window_days: i64,
    pub visible: bool,
    pub floor_value: f64,
    pub locale: String,
    pub active: bool,
    pub status: AuditEventEntryStatus,
}

impl AuditEventEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventEntryStatus) -> bool {
        match self.status {
            AuditEventEntryStatus::Open => {
                match next {
                    AuditEventEntryStatus::Resolved | AuditEventEntryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventEntryStatus::InReview => {
                match next {
                    AuditEventEntryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventEntryStatus::Resolved => {
                match next {
                    AuditEventEntryStatus::Reopened | AuditEventEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventEntryStatus::Reopened => {
                match next {
                    AuditEventEntryStatus::Archived => {
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

    pub fn move_to(&mut self, next: AuditEventEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.floor_value - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventEntry {
        AuditEventEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            cost: self.cost,
            priority: self.priority,
            namespace: self.namespace.clone(),
            window_days: self.window_days,
            visible: self.visible,
            floor_value: self.floor_value,
            locale: self.locale.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a audit event entry in status open.
pub fn new_audit_event_entry(code: &str, name: &str) -> AuditEventEntry {
    AuditEventEntry {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        cost: 0.0,
        priority: 0,
        namespace: "".to_string(),
        window_days: 0,
        visible: false,
        floor_value: 0.0,
        locale: "".to_string(),
        active: false,
        status: AuditEventEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_audit_event_entry("code-660", "name-920");
        assert_eq!(item.status, AuditEventEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_audit_event_entry("code-660", "name-920");
        if item.move_to(AuditEventEntryStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventEntryStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_audit_event_entry("code-660", "name-920");
        item.status = AuditEventEntryStatus::Archived;
        assert!(
            item.move_to(AuditEventEntryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
