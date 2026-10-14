use super::*;

use acme_kit::text;

/// Where a legacy audit event schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAuditEventScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyAuditEventScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyAuditEventScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy audit event schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAuditEventSchedule {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub port: i64,
    pub tags: Vec<String>,
    pub status: LegacyAuditEventScheduleStatus,
}

impl LegacyAuditEventSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAuditEventScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAuditEventScheduleStatus) -> bool {
        match self.status {
            LegacyAuditEventScheduleStatus::Requested => {
                match next {
                    LegacyAuditEventScheduleStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAuditEventScheduleStatus::Granted => {
                return next == LegacyAuditEventScheduleStatus::Expired;
            }
            LegacyAuditEventScheduleStatus::Revoked => {
                return next == LegacyAuditEventScheduleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyAuditEventScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy audit event schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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
        self.port as f64
    }

    pub fn copy(&self) -> LegacyAuditEventSchedule {
        LegacyAuditEventSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            port: self.port,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy audit event schedule in status requested.
pub fn new_legacy_audit_event_schedule(code: &str, title: &str) -> LegacyAuditEventSchedule {
    LegacyAuditEventSchedule {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        port: 0,
        tags: Vec::new(),
        status: LegacyAuditEventScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_audit_event_schedule("code-332", "title-895");
        assert_eq!(item.status, LegacyAuditEventScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_legacy_audit_event_schedule("code-332", "title-895");
        if item.move_to(LegacyAuditEventScheduleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyAuditEventScheduleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_audit_event_schedule("code-332", "title-895");
        item.status = LegacyAuditEventScheduleStatus::Expired;
        assert!(
            item.move_to(LegacyAuditEventScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
