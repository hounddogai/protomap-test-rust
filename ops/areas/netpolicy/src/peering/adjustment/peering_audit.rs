use super::*;

use acme_kit::text;

/// Where a peering audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PeeringAuditStatus {
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

impl std::fmt::Display for PeeringAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering audit as the peering team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAudit {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub notes: String,
    pub maximum: f64,
    pub status: PeeringAuditStatus,
}

impl PeeringAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringAuditStatus) -> bool {
        match self.status {
            PeeringAuditStatus::Open => {
                return next == PeeringAuditStatus::InReview;
            }
            PeeringAuditStatus::InReview => {
                return next == PeeringAuditStatus::Reopened || next == PeeringAuditStatus::Archived;
            }
            PeeringAuditStatus::Resolved => {
                match next {
                    PeeringAuditStatus::Reopened | PeeringAuditStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringAuditStatus::Reopened => {
                return next == PeeringAuditStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> PeeringAudit {
        PeeringAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            notes: self.notes.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a peering audit in status open.
pub fn new_peering_audit(code: &str, title: &str) -> PeeringAudit {
    PeeringAudit {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        notes: "".to_string(),
        maximum: 0.0,
        status: PeeringAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_peering_audit("code-209", "title-321");
        assert_eq!(item.status, PeeringAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_peering_audit("code-209", "title-321");
        if item.move_to(PeeringAuditStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringAuditStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_peering_audit("code-209", "title-321");
        item.status = PeeringAuditStatus::Archived;
        assert!(item.move_to(PeeringAuditStatus::Open).is_err(), "expected move to fail");
    }
}
