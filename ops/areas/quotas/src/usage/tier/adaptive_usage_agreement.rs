use super::*;

use acme_kit::text;

/// Where a adaptive usage agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveUsageAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveUsageAgreementStatus {
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

impl std::fmt::Display for AdaptiveUsageAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive usage agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveUsageAgreement {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub attempts: i64,
    pub host: String,
    pub pinned: bool,
    pub fingerprint: String,
    pub label: String,
    pub scope: String,
    pub minimum: f64,
    pub status: AdaptiveUsageAgreementStatus,
}

impl AdaptiveUsageAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveUsageAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveUsageAgreementStatus) -> bool {
        match self.status {
            AdaptiveUsageAgreementStatus::Draft => {
                return next == AdaptiveUsageAgreementStatus::Suspended;
            }
            AdaptiveUsageAgreementStatus::Pending => {
                return next == AdaptiveUsageAgreementStatus::Active;
            }
            AdaptiveUsageAgreementStatus::Active => {
                match next {
                    AdaptiveUsageAgreementStatus::Suspended | AdaptiveUsageAgreementStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveUsageAgreementStatus::Suspended => {
                match next {
                    AdaptiveUsageAgreementStatus::Closed => {
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

    pub fn move_to(&mut self, next: AdaptiveUsageAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive usage agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive usage agreement", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.minimum - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveUsageAgreement {
        AdaptiveUsageAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            attempts: self.attempts,
            host: self.host.clone(),
            pinned: self.pinned,
            fingerprint: self.fingerprint.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive usage agreement in status draft.
pub fn new_adaptive_usage_agreement(code: &str, title: &str) -> AdaptiveUsageAgreement {
    AdaptiveUsageAgreement {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        attempts: 0,
        host: "".to_string(),
        pinned: false,
        fingerprint: "".to_string(),
        label: "".to_string(),
        scope: "".to_string(),
        minimum: 0.0,
        status: AdaptiveUsageAgreementStatus::Draft,
    }
}
