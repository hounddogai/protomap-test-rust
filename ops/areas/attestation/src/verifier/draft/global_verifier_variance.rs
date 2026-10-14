use super::*;

use acme_kit::text;

/// Where a global verifier variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalVerifierVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalVerifierVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalVerifierVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global verifier variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalVerifierVariance {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub fingerprint: String,
    pub window_days: i64,
    pub maximum: f64,
    pub principal: String,
    pub status: GlobalVerifierVarianceStatus,
}

impl GlobalVerifierVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalVerifierVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalVerifierVarianceStatus) -> bool {
        match self.status {
            GlobalVerifierVarianceStatus::Proposed => {
                return next == GlobalVerifierVarianceStatus::Approved;
            }
            GlobalVerifierVarianceStatus::Approved => {
                match next {
                    GlobalVerifierVarianceStatus::Rejected | GlobalVerifierVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalVerifierVarianceStatus::Rejected => {
                return next == GlobalVerifierVarianceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalVerifierVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global verifier variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global verifier variance", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.weight - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalVerifierVariance {
        GlobalVerifierVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            fingerprint: self.fingerprint.clone(),
            window_days: self.window_days,
            maximum: self.maximum,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global verifier variance in status proposed.
pub fn new_global_verifier_variance(code: &str, label: &str) -> GlobalVerifierVariance {
    GlobalVerifierVariance {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        fingerprint: "".to_string(),
        window_days: 0,
        maximum: 0.0,
        principal: "".to_string(),
        status: GlobalVerifierVarianceStatus::Proposed,
    }
}
