use super::*;

use acme_kit::text;

/// Where a scheduled audit event record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledAuditEventRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledAuditEventRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledAuditEventRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled audit event record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventRecord {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub revision: i64,
    pub score: f64,
    pub contact_email: String,
    pub status: ScheduledAuditEventRecordStatus,
}

impl ScheduledAuditEventRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledAuditEventRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledAuditEventRecordStatus) -> bool {
        match self.status {
            ScheduledAuditEventRecordStatus::Created => {
                match next {
                    ScheduledAuditEventRecordStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledAuditEventRecordStatus::Validated => {
                return next == ScheduledAuditEventRecordStatus::Published;
            }
            ScheduledAuditEventRecordStatus::Published => {
                match next {
                    ScheduledAuditEventRecordStatus::Retired => {
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

    pub fn move_to(&mut self, next: ScheduledAuditEventRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled audit event record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        (self.score * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledAuditEventRecord {
        ScheduledAuditEventRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            revision: self.revision,
            score: self.score,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled audit event record in status created.
pub fn new_scheduled_audit_event_record(code: &str, title: &str) -> ScheduledAuditEventRecord {
    ScheduledAuditEventRecord {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        revision: 0,
        score: 0.0,
        contact_email: "".to_string(),
        status: ScheduledAuditEventRecordStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_audit_event_record("code-610", "title-658");
        assert_eq!(item.status, ScheduledAuditEventRecordStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_scheduled_audit_event_record("code-610", "title-658");
        if item.move_to(ScheduledAuditEventRecordStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledAuditEventRecordStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_audit_event_record("code-610", "title-658");
        item.status = ScheduledAuditEventRecordStatus::Retired;
        assert!(
            item.move_to(ScheduledAuditEventRecordStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
