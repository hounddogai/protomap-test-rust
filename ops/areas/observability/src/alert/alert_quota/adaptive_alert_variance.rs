use super::*;

use acme_kit::text;

/// Where a adaptive alert variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveAlertVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveAlertVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for AdaptiveAlertVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive alert variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertVariance {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub notes: String,
    pub window_days: i64,
    pub weight: f64,
    pub contact_email: String,
    pub created_at: i64,
    pub burst: i64,
    pub status: AdaptiveAlertVarianceStatus,
}

impl AdaptiveAlertVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveAlertVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveAlertVarianceStatus) -> bool {
        match self.status {
            AdaptiveAlertVarianceStatus::New => {
                return next == AdaptiveAlertVarianceStatus::Queued || next == AdaptiveAlertVarianceStatus::Failed;
            }
            AdaptiveAlertVarianceStatus::Queued => {
                return next == AdaptiveAlertVarianceStatus::Failed;
            }
            AdaptiveAlertVarianceStatus::Running => {
                return next == AdaptiveAlertVarianceStatus::Done || next == AdaptiveAlertVarianceStatus::Failed;
            }
            AdaptiveAlertVarianceStatus::Done => {
                return next == AdaptiveAlertVarianceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveAlertVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive alert variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive alert variance", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.weight - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveAlertVariance {
        AdaptiveAlertVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            notes: self.notes.clone(),
            window_days: self.window_days,
            weight: self.weight,
            contact_email: self.contact_email.clone(),
            created_at: self.created_at,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive alert variance in status new.
pub fn new_adaptive_alert_variance(code: &str, title: &str) -> AdaptiveAlertVariance {
    AdaptiveAlertVariance {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        notes: "".to_string(),
        window_days: 0,
        weight: 0.0,
        contact_email: "".to_string(),
        created_at: 0,
        burst: 0,
        status: AdaptiveAlertVarianceStatus::New,
    }
}
