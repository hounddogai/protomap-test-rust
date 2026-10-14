use super::*;

use acme_kit::text;

/// Where a audit event checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AuditEventCheckpointStatus {
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

impl std::fmt::Display for AuditEventCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A audit event checkpoint as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCheckpoint {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub amount: f64,
    pub unit: String,
    pub rate: f64,
    pub status: AuditEventCheckpointStatus,
}

impl AuditEventCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventCheckpointStatus) -> bool {
        match self.status {
            AuditEventCheckpointStatus::Open => {
                return next == AuditEventCheckpointStatus::InReview || next == AuditEventCheckpointStatus::Resolved;
            }
            AuditEventCheckpointStatus::InReview => {
                return next == AuditEventCheckpointStatus::Resolved || next == AuditEventCheckpointStatus::Archived;
            }
            AuditEventCheckpointStatus::Resolved => {
                match next {
                    AuditEventCheckpointStatus::Reopened | AuditEventCheckpointStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventCheckpointStatus::Reopened => {
                match next {
                    AuditEventCheckpointStatus::Archived => {
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

    pub fn move_to(&mut self, next: AuditEventCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.rate * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventCheckpoint {
        AuditEventCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            amount: self.amount,
            unit: self.unit.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a audit event checkpoint in status open.
pub fn new_audit_event_checkpoint(code: &str, title: &str) -> AuditEventCheckpoint {
    AuditEventCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        amount: 0.0,
        unit: "".to_string(),
        rate: 0.0,
        status: AuditEventCheckpointStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_audit_event_checkpoint("code-974", "title-652");
        assert_eq!(item.status, AuditEventCheckpointStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_audit_event_checkpoint("code-974", "title-652");
        if item.move_to(AuditEventCheckpointStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventCheckpointStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_audit_event_checkpoint("code-974", "title-652");
        item.status = AuditEventCheckpointStatus::Archived;
        assert!(
            item.move_to(AuditEventCheckpointStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
