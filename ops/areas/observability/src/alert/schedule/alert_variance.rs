use super::*;

use acme_kit::text;

/// Where a alert variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AlertVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AlertVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a alert variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertVariance {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub score: f64,
    pub title: String,
    pub contact_email: String,
    pub country: String,
    pub status: AlertVarianceStatus,
}

impl AlertVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertVarianceStatus) -> bool {
        match self.status {
            AlertVarianceStatus::Proposed => {
                match next {
                    AlertVarianceStatus::Approved | AlertVarianceStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertVarianceStatus::Approved => {
                return next == AlertVarianceStatus::Rejected || next == AlertVarianceStatus::Withdrawn;
            }
            AlertVarianceStatus::Rejected => {
                return next == AlertVarianceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.score * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertVariance {
        AlertVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            score: self.score,
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a alert variance in status proposed.
pub fn new_alert_variance(code: &str, name: &str) -> AlertVariance {
    AlertVariance {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        score: 0.0,
        title: "".to_string(),
        contact_email: "".to_string(),
        country: "".to_string(),
        status: AlertVarianceStatus::Proposed,
    }
}
