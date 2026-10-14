use super::*;

use acme_kit::text;

/// Where a fallback grant agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackGrantAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackGrantAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackGrantAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback grant agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackGrantAgreement {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub duration_minutes: i64,
    pub country: String,
    pub burst: i64,
    pub status: FallbackGrantAgreementStatus,
}

impl FallbackGrantAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackGrantAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackGrantAgreementStatus) -> bool {
        match self.status {
            FallbackGrantAgreementStatus::Requested => {
                return next == FallbackGrantAgreementStatus::Expired;
            }
            FallbackGrantAgreementStatus::Granted => {
                return next == FallbackGrantAgreementStatus::Revoked || next == FallbackGrantAgreementStatus::Expired;
            }
            FallbackGrantAgreementStatus::Revoked => {
                match next {
                    FallbackGrantAgreementStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackGrantAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback grant agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.burst as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackGrantAgreement {
        FallbackGrantAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            duration_minutes: self.duration_minutes,
            country: self.country.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a fallback grant agreement in status requested.
pub fn new_fallback_grant_agreement(code: &str, label: &str) -> FallbackGrantAgreement {
    FallbackGrantAgreement {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        duration_minutes: 0,
        country: "".to_string(),
        burst: 0,
        status: FallbackGrantAgreementStatus::Requested,
    }
}
