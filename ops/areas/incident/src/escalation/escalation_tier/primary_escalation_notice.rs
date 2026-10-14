use super::*;

use acme_kit::text;

/// Where a primary escalation notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryEscalationNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryEscalationNoticeStatus {
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

impl std::fmt::Display for PrimaryEscalationNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary escalation notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEscalationNotice {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub label: String,
    pub scope: String,
    pub status: PrimaryEscalationNoticeStatus,
}

impl PrimaryEscalationNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryEscalationNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryEscalationNoticeStatus) -> bool {
        match self.status {
            PrimaryEscalationNoticeStatus::Open => {
                match next {
                    PrimaryEscalationNoticeStatus::InReview | PrimaryEscalationNoticeStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEscalationNoticeStatus::InReview => {
                match next {
                    PrimaryEscalationNoticeStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryEscalationNoticeStatus::Resolved => {
                return next == PrimaryEscalationNoticeStatus::Reopened
                    || next == PrimaryEscalationNoticeStatus::Archived;
            }
            PrimaryEscalationNoticeStatus::Reopened => {
                return next == PrimaryEscalationNoticeStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryEscalationNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary escalation notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary escalation notice", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> PrimaryEscalationNotice {
        PrimaryEscalationNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            label: self.label.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a primary escalation notice in status open.
pub fn new_primary_escalation_notice(code: &str, name: &str) -> PrimaryEscalationNotice {
    PrimaryEscalationNotice {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        label: "".to_string(),
        scope: "".to_string(),
        status: PrimaryEscalationNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_escalation_notice("code-351", "name-897");
        assert_eq!(item.status, PrimaryEscalationNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_primary_escalation_notice("code-351", "name-897");
        if item.move_to(PrimaryEscalationNoticeStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryEscalationNoticeStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_escalation_notice("code-351", "name-897");
        item.status = PrimaryEscalationNoticeStatus::Archived;
        assert!(
            item.move_to(PrimaryEscalationNoticeStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
