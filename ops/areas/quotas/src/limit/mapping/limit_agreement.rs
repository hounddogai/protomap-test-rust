use super::*;

use acme_kit::text;

/// Where a limit agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LimitAgreementStatus {
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

impl std::fmt::Display for LimitAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a limit agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitAgreement {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub attempts: i64,
    pub ratio: f64,
    pub status: LimitAgreementStatus,
}

impl LimitAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitAgreementStatus) -> bool {
        match self.status {
            LimitAgreementStatus::Draft => {
                return next == LimitAgreementStatus::Pending;
            }
            LimitAgreementStatus::Pending => {
                return next == LimitAgreementStatus::Closed;
            }
            LimitAgreementStatus::Active => {
                match next {
                    LimitAgreementStatus::Suspended | LimitAgreementStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitAgreementStatus::Suspended => {
                match next {
                    LimitAgreementStatus::Closed => {
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

    pub fn move_to(&mut self, next: LimitAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.cost - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitAgreement {
        LimitAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            attempts: self.attempts,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a limit agreement in status draft.
pub fn new_limit_agreement(code: &str, title: &str) -> LimitAgreement {
    LimitAgreement {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        attempts: 0,
        ratio: 0.0,
        status: LimitAgreementStatus::Draft,
    }
}
