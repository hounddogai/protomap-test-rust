use super::*;

use acme_kit::text;

/// Where a adaptive forecast score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveForecastScoreStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveForecastScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for AdaptiveForecastScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive forecast score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveForecastScore {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub timezone: String,
    pub ceiling: f64,
    pub cipher: String,
    pub locale: String,
    pub scope: String,
    pub amount: f64,
    pub issuer: String,
    pub status: AdaptiveForecastScoreStatus,
}

impl AdaptiveForecastScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveForecastScoreStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveForecastScoreStatus) -> bool {
        match self.status {
            AdaptiveForecastScoreStatus::Planned => {
                match next {
                    AdaptiveForecastScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveForecastScoreStatus::Scheduled => {
                match next {
                    AdaptiveForecastScoreStatus::InProgress | AdaptiveForecastScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveForecastScoreStatus::InProgress => {
                match next {
                    AdaptiveForecastScoreStatus::Completed | AdaptiveForecastScoreStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveForecastScoreStatus::Completed => {
                return next == AdaptiveForecastScoreStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveForecastScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive forecast score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ratio - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveForecastScore {
        AdaptiveForecastScore {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            timezone: self.timezone.clone(),
            ceiling: self.ceiling,
            cipher: self.cipher.clone(),
            locale: self.locale.clone(),
            scope: self.scope.clone(),
            amount: self.amount,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a adaptive forecast score in status planned.
pub fn new_adaptive_forecast_score(code: &str, name: &str) -> AdaptiveForecastScore {
    AdaptiveForecastScore {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        timezone: "".to_string(),
        ceiling: 0.0,
        cipher: "".to_string(),
        locale: "".to_string(),
        scope: "".to_string(),
        amount: 0.0,
        issuer: "".to_string(),
        status: AdaptiveForecastScoreStatus::Planned,
    }
}
