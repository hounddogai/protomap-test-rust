use super::*;

use acme_kit::text;

/// Where a audit event review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AuditEventReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AuditEventReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventReview {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub maximum: f64,
    pub segment: String,
    pub status: AuditEventReviewStatus,
}

impl AuditEventReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventReviewStatus) -> bool {
        match self.status {
            AuditEventReviewStatus::Proposed => {
                match next {
                    AuditEventReviewStatus::Approved | AuditEventReviewStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventReviewStatus::Approved => {
                return next == AuditEventReviewStatus::Rejected;
            }
            AuditEventReviewStatus::Rejected => {
                match next {
                    AuditEventReviewStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AuditEventReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> AuditEventReview {
        AuditEventReview {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            maximum: self.maximum,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a audit event review in status proposed.
pub fn new_audit_event_review(code: &str, name: &str) -> AuditEventReview {
    AuditEventReview {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        maximum: 0.0,
        segment: "".to_string(),
        status: AuditEventReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_audit_event_review("code-616", "name-672");
        assert_eq!(item.status, AuditEventReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_audit_event_review("code-616", "name-672");
        if item.move_to(AuditEventReviewStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventReviewStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_audit_event_review("code-616", "name-672");
        item.status = AuditEventReviewStatus::Withdrawn;
        assert!(
            item.move_to(AuditEventReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
