use super::*;

use acme_kit::text;

/// Where a approval agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ApprovalAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ApprovalAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a approval agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAgreement {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub country: String,
    pub batch_size: i64,
    pub name: String,
    pub principal: String,
    pub contact_email: String,
    pub score: f64,
    pub status: ApprovalAgreementStatus,
}

impl ApprovalAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalAgreementStatus) -> bool {
        match self.status {
            ApprovalAgreementStatus::Requested => {
                match next {
                    ApprovalAgreementStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalAgreementStatus::Granted => {
                return next == ApprovalAgreementStatus::Expired;
            }
            ApprovalAgreementStatus::Revoked => {
                return next == ApprovalAgreementStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ApprovalAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.batch_size as f64) * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalAgreement {
        ApprovalAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            country: self.country.clone(),
            batch_size: self.batch_size,
            name: self.name.clone(),
            principal: self.principal.clone(),
            contact_email: self.contact_email.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a approval agreement in status requested.
pub fn new_approval_agreement(code: &str, title: &str) -> ApprovalAgreement {
    ApprovalAgreement {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        country: "".to_string(),
        batch_size: 0,
        name: "".to_string(),
        principal: "".to_string(),
        contact_email: "".to_string(),
        score: 0.0,
        status: ApprovalAgreementStatus::Requested,
    }
}
