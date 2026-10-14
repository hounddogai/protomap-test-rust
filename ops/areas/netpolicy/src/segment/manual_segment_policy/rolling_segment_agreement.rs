use super::*;

use acme_kit::text;

/// Where a rolling segment agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSegmentAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingSegmentAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingSegmentAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling segment agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentAgreement {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub archived: bool,
    pub quantity: i64,
    pub name: String,
    pub external_ref: String,
    pub scope: String,
    pub ceiling: f64,
    pub issuer: String,
    pub weight: f64,
    pub status: RollingSegmentAgreementStatus,
}

impl RollingSegmentAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSegmentAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSegmentAgreementStatus) -> bool {
        match self.status {
            RollingSegmentAgreementStatus::Requested => {
                match next {
                    RollingSegmentAgreementStatus::Revoked | RollingSegmentAgreementStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSegmentAgreementStatus::Granted => {
                return next == RollingSegmentAgreementStatus::Expired;
            }
            RollingSegmentAgreementStatus::Revoked => {
                return next == RollingSegmentAgreementStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingSegmentAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling segment agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.weight - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSegmentAgreement {
        RollingSegmentAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            archived: self.archived,
            quantity: self.quantity,
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            issuer: self.issuer.clone(),
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rolling segment agreement in status requested.
pub fn new_rolling_segment_agreement(code: &str, title: &str) -> RollingSegmentAgreement {
    RollingSegmentAgreement {
        code: code.to_string(),
        title: title.to_string(),
        contact_email: "".to_string(),
        archived: false,
        quantity: 0,
        name: "".to_string(),
        external_ref: "".to_string(),
        scope: "".to_string(),
        ceiling: 0.0,
        issuer: "".to_string(),
        weight: 0.0,
        status: RollingSegmentAgreementStatus::Requested,
    }
}
