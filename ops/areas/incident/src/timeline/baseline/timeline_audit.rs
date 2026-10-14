use super::*;

use acme_kit::text;

/// Where a timeline audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TimelineAuditStatus {
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

impl std::fmt::Display for TimelineAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineAudit {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub revision: i64,
    pub contact_email: String,
    pub quota_units: i64,
    pub status: TimelineAuditStatus,
}

impl TimelineAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineAuditStatus) -> bool {
        match self.status {
            TimelineAuditStatus::Draft => {
                return next == TimelineAuditStatus::Suspended || next == TimelineAuditStatus::Closed;
            }
            TimelineAuditStatus::Pending => {
                match next {
                    TimelineAuditStatus::Suspended | TimelineAuditStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineAuditStatus::Active => {
                return next == TimelineAuditStatus::Suspended || next == TimelineAuditStatus::Closed;
            }
            TimelineAuditStatus::Suspended => {
                match next {
                    TimelineAuditStatus::Closed => {
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

    pub fn move_to(&mut self, next: TimelineAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.revision as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineAudit {
        TimelineAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            revision: self.revision,
            contact_email: self.contact_email.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a timeline audit in status draft.
pub fn new_timeline_audit(code: &str, name: &str) -> TimelineAudit {
    TimelineAudit {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        revision: 0,
        contact_email: "".to_string(),
        quota_units: 0,
        status: TimelineAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_timeline_audit("code-989", "name-790");
        assert_eq!(item.status, TimelineAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_timeline_audit("code-989", "name-790");
        if item.move_to(TimelineAuditStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineAuditStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_timeline_audit("code-989", "name-790");
        item.status = TimelineAuditStatus::Closed;
        assert!(
            item.move_to(TimelineAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
