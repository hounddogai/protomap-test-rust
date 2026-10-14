use super::*;

use acme_kit::text;

/// Where a seasonal allowlist forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalAllowlistForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalAllowlistForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalAllowlistForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal allowlist forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistForecast {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub country: String,
    pub score: f64,
    pub burst: i64,
    pub summary: String,
    pub status: SeasonalAllowlistForecastStatus,
}

impl SeasonalAllowlistForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalAllowlistForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalAllowlistForecastStatus) -> bool {
        match self.status {
            SeasonalAllowlistForecastStatus::Proposed => {
                match next {
                    SeasonalAllowlistForecastStatus::Rejected | SeasonalAllowlistForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAllowlistForecastStatus::Approved => {
                return next == SeasonalAllowlistForecastStatus::Rejected;
            }
            SeasonalAllowlistForecastStatus::Rejected => {
                match next {
                    SeasonalAllowlistForecastStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SeasonalAllowlistForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal allowlist forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.burst as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalAllowlistForecast {
        SeasonalAllowlistForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            country: self.country.clone(),
            score: self.score,
            burst: self.burst,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal allowlist forecast in status proposed.
pub fn new_seasonal_allowlist_forecast(code: &str, title: &str) -> SeasonalAllowlistForecast {
    SeasonalAllowlistForecast {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        country: "".to_string(),
        score: 0.0,
        burst: 0,
        summary: "".to_string(),
        status: SeasonalAllowlistForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_allowlist_forecast("code-289", "title-93");
        assert_eq!(item.status, SeasonalAllowlistForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_seasonal_allowlist_forecast("code-289", "title-93");
        if item.move_to(SeasonalAllowlistForecastStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalAllowlistForecastStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_allowlist_forecast("code-289", "title-93");
        item.status = SeasonalAllowlistForecastStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalAllowlistForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
