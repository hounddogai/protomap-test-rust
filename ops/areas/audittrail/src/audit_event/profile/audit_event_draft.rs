use super::*;

use acme_kit::text;

/// Where a audit event draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AuditEventDraftStatus {
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

impl std::fmt::Display for AuditEventDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventDraft {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub active: bool,
    pub currency: String,
    pub priority: i64,
    pub port: i64,
    pub capacity: i64,
    pub visible: bool,
    pub margin: f64,
    pub status: AuditEventDraftStatus,
}

impl AuditEventDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventDraftStatus) -> bool {
        match self.status {
            AuditEventDraftStatus::Open => {
                match next {
                    AuditEventDraftStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventDraftStatus::InReview => {
                match next {
                    AuditEventDraftStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventDraftStatus::Resolved => {
                return next == AuditEventDraftStatus::Reopened || next == AuditEventDraftStatus::Archived;
            }
            AuditEventDraftStatus::Reopened => {
                match next {
                    AuditEventDraftStatus::Archived => {
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

    pub fn move_to(&mut self, next: AuditEventDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.priority as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventDraft {
        AuditEventDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            active: self.active,
            currency: self.currency.clone(),
            priority: self.priority,
            port: self.port,
            capacity: self.capacity,
            visible: self.visible,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a audit event draft in status open.
pub fn new_audit_event_draft(code: &str, name: &str) -> AuditEventDraft {
    AuditEventDraft {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        active: false,
        currency: "".to_string(),
        priority: 0,
        port: 0,
        capacity: 0,
        visible: false,
        margin: 0.0,
        status: AuditEventDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_audit_event_draft("code-729", "name-336");
        assert_eq!(item.status, AuditEventDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_audit_event_draft("code-729", "name-336");
        if item.move_to(AuditEventDraftStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventDraftStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_audit_event_draft("code-729", "name-336");
        item.status = AuditEventDraftStatus::Archived;
        assert!(
            item.move_to(AuditEventDraftStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
