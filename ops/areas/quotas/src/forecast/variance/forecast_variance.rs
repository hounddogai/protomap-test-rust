use super::*;

use acme_kit::text;

/// Where a forecast variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastVarianceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ForecastVarianceStatus {
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

impl std::fmt::Display for ForecastVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast variance as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastVariance {
    pub code: String,
    pub title: String,
    pub floor_value: f64,
    pub approved: bool,
    pub fingerprint: String,
    pub cost: f64,
    pub status: ForecastVarianceStatus,
}

impl ForecastVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastVarianceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastVarianceStatus) -> bool {
        match self.status {
            ForecastVarianceStatus::Open => {
                match next {
                    ForecastVarianceStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastVarianceStatus::InReview => {
                match next {
                    ForecastVarianceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastVarianceStatus::Resolved => {
                match next {
                    ForecastVarianceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastVarianceStatus::Reopened => {
                return next == ForecastVarianceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.cost - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastVariance {
        ForecastVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            floor_value: self.floor_value,
            approved: self.approved,
            fingerprint: self.fingerprint.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a forecast variance in status open.
pub fn new_forecast_variance(code: &str, title: &str) -> ForecastVariance {
    ForecastVariance {
        code: code.to_string(),
        title: title.to_string(),
        floor_value: 0.0,
        approved: false,
        fingerprint: "".to_string(),
        cost: 0.0,
        status: ForecastVarianceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_forecast_variance("code-883", "title-241");
        assert_eq!(item.status, ForecastVarianceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_forecast_variance("code-883", "title-241");
        if item.move_to(ForecastVarianceStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastVarianceStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_forecast_variance("code-883", "title-241");
        item.status = ForecastVarianceStatus::Archived;
        assert!(
            item.move_to(ForecastVarianceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
