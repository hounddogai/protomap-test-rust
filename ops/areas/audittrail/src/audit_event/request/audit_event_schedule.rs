use super::*;

use acme_kit::text;

/// Where a audit event schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AuditEventScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AuditEventScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSchedule {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub visible: bool,
    pub grace_days: i64,
    pub timezone: String,
    pub region: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub updated_at: i64,
    pub rotation_days: i64,
    pub status: AuditEventScheduleStatus,
}

impl AuditEventSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventScheduleStatus) -> bool {
        match self.status {
            AuditEventScheduleStatus::Proposed => {
                match next {
                    AuditEventScheduleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventScheduleStatus::Approved => {
                return next == AuditEventScheduleStatus::Rejected || next == AuditEventScheduleStatus::Withdrawn;
            }
            AuditEventScheduleStatus::Rejected => {
                match next {
                    AuditEventScheduleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AuditEventScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.updated_at as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventSchedule {
        AuditEventSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            visible: self.visible,
            grace_days: self.grace_days,
            timezone: self.timezone.clone(),
            region: self.region.clone(),
            tags: self.tags.clone(),
            pinned: self.pinned,
            updated_at: self.updated_at,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a audit event schedule in status proposed.
pub fn new_audit_event_schedule(code: &str, name: &str) -> AuditEventSchedule {
    AuditEventSchedule {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        visible: false,
        grace_days: 0,
        timezone: "".to_string(),
        region: "".to_string(),
        tags: Vec::new(),
        pinned: false,
        updated_at: 0,
        rotation_days: 0,
        status: AuditEventScheduleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_audit_event_schedule("code-390", "name-732");
        assert_eq!(item.status, AuditEventScheduleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_audit_event_schedule("code-390", "name-732");
        if item.move_to(AuditEventScheduleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventScheduleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_audit_event_schedule("code-390", "name-732");
        item.status = AuditEventScheduleStatus::Withdrawn;
        assert!(
            item.move_to(AuditEventScheduleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
