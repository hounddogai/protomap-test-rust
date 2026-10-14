use super::*;

use acme_kit::text;

/// Where a health check agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl HealthCheckAgreementStatus {
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

impl std::fmt::Display for HealthCheckAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAgreement {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub reviewer: String,
    pub category: String,
    pub attempts: i64,
    pub pinned: bool,
    pub ceiling: f64,
    pub status: HealthCheckAgreementStatus,
}

impl HealthCheckAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckAgreementStatus) -> bool {
        match self.status {
            HealthCheckAgreementStatus::Open => {
                return next == HealthCheckAgreementStatus::InReview || next == HealthCheckAgreementStatus::Resolved;
            }
            HealthCheckAgreementStatus::InReview => {
                match next {
                    HealthCheckAgreementStatus::Reopened | HealthCheckAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckAgreementStatus::Resolved => {
                match next {
                    HealthCheckAgreementStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckAgreementStatus::Reopened => {
                return next == HealthCheckAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "health check agreement", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.attempts as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckAgreement {
        HealthCheckAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            reviewer: self.reviewer.clone(),
            category: self.category.clone(),
            attempts: self.attempts,
            pinned: self.pinned,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a health check agreement in status open.
pub fn new_health_check_agreement(code: &str, title: &str) -> HealthCheckAgreement {
    HealthCheckAgreement {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        reviewer: "".to_string(),
        category: "".to_string(),
        attempts: 0,
        pinned: false,
        ceiling: 0.0,
        status: HealthCheckAgreementStatus::Open,
    }
}
