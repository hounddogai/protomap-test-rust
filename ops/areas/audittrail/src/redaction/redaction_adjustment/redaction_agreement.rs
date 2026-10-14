use super::*;

use acme_kit::text;

/// Where a redaction agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionAgreementStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RedactionAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RedactionAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a redaction agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAgreement {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub approved: bool,
    pub currency: String,
    pub duration_minutes: i64,
    pub host: String,
    pub amount: f64,
    pub active: bool,
    pub ceiling: f64,
    pub ttl_seconds: i64,
    pub status: RedactionAgreementStatus,
}

impl RedactionAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionAgreementStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionAgreementStatus) -> bool {
        match self.status {
            RedactionAgreementStatus::Planned => {
                return next == RedactionAgreementStatus::InProgress;
            }
            RedactionAgreementStatus::Scheduled => {
                return next == RedactionAgreementStatus::Cancelled;
            }
            RedactionAgreementStatus::InProgress => {
                match next {
                    RedactionAgreementStatus::Completed | RedactionAgreementStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionAgreementStatus::Completed => {
                match next {
                    RedactionAgreementStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RedactionAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "redaction agreement", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionAgreement {
        RedactionAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            approved: self.approved,
            currency: self.currency.clone(),
            duration_minutes: self.duration_minutes,
            host: self.host.clone(),
            amount: self.amount,
            active: self.active,
            ceiling: self.ceiling,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a redaction agreement in status planned.
pub fn new_redaction_agreement(code: &str, name: &str) -> RedactionAgreement {
    RedactionAgreement {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        approved: false,
        currency: "".to_string(),
        duration_minutes: 0,
        host: "".to_string(),
        amount: 0.0,
        active: false,
        ceiling: 0.0,
        ttl_seconds: 0,
        status: RedactionAgreementStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_redaction_agreement("code-393", "name-385");
        assert_eq!(item.status, RedactionAgreementStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_redaction_agreement("code-393", "name-385");
        if item.move_to(RedactionAgreementStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionAgreementStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_redaction_agreement("code-393", "name-385");
        item.status = RedactionAgreementStatus::Cancelled;
        assert!(
            item.move_to(RedactionAgreementStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
