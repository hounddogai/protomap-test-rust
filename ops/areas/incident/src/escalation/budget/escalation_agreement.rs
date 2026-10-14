use super::*;

use acme_kit::text;

/// Where a escalation agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EscalationAgreementStatus {
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

impl std::fmt::Display for EscalationAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAgreement {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub maximum: f64,
    pub expires_at: i64,
    pub quantity: i64,
    pub label: String,
    pub status: EscalationAgreementStatus,
}

impl EscalationAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationAgreementStatus) -> bool {
        match self.status {
            EscalationAgreementStatus::Open => {
                return next == EscalationAgreementStatus::Reopened || next == EscalationAgreementStatus::Archived;
            }
            EscalationAgreementStatus::InReview => {
                match next {
                    EscalationAgreementStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationAgreementStatus::Resolved => {
                return next == EscalationAgreementStatus::Reopened;
            }
            EscalationAgreementStatus::Reopened => {
                match next {
                    EscalationAgreementStatus::Archived => {
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

    pub fn move_to(&mut self, next: EscalationAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "escalation agreement", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.burst as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationAgreement {
        EscalationAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            maximum: self.maximum,
            expires_at: self.expires_at,
            quantity: self.quantity,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a escalation agreement in status open.
pub fn new_escalation_agreement(code: &str, name: &str) -> EscalationAgreement {
    EscalationAgreement {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        maximum: 0.0,
        expires_at: 0,
        quantity: 0,
        label: "".to_string(),
        status: EscalationAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_escalation_agreement("code-935", "name-983");
        assert_eq!(item.status, EscalationAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_escalation_agreement("code-935", "name-983");
        if item.move_to(EscalationAgreementStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationAgreementStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_escalation_agreement("code-935", "name-983");
        item.status = EscalationAgreementStatus::Archived;
        assert!(
            item.move_to(EscalationAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
