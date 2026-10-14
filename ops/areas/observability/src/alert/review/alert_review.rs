use super::*;

use acme_kit::text;

/// Where a alert review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertReviewStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AlertReviewStatus {
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

impl std::fmt::Display for AlertReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertReview {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub batch_size: i64,
    pub rotation_days: i64,
    pub version: i64,
    pub status: AlertReviewStatus,
}

impl AlertReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertReviewStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertReviewStatus) -> bool {
        match self.status {
            AlertReviewStatus::Draft => {
                return next == AlertReviewStatus::Active;
            }
            AlertReviewStatus::Pending => {
                return next == AlertReviewStatus::Active || next == AlertReviewStatus::Closed;
            }
            AlertReviewStatus::Active => {
                match next {
                    AlertReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertReviewStatus::Suspended => {
                return next == AlertReviewStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AlertReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "alert review", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.batch_size as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertReview {
        AlertReview {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            batch_size: self.batch_size,
            rotation_days: self.rotation_days,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a alert review in status draft.
pub fn new_alert_review(code: &str, name: &str) -> AlertReview {
    AlertReview {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        batch_size: 0,
        rotation_days: 0,
        version: 0,
        status: AlertReviewStatus::Draft,
    }
}
