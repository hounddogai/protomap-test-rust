use super::*;

use acme_kit::text;

/// Where a seasonal page forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalPageForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalPageForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalPageForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal page forecast as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageForecast {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub amount: f64,
    pub owner_id: String,
    pub margin: f64,
    pub status: SeasonalPageForecastStatus,
}

impl SeasonalPageForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalPageForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalPageForecastStatus) -> bool {
        match self.status {
            SeasonalPageForecastStatus::Requested => {
                return next == SeasonalPageForecastStatus::Expired;
            }
            SeasonalPageForecastStatus::Granted => {
                match next {
                    SeasonalPageForecastStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalPageForecastStatus::Revoked => {
                match next {
                    SeasonalPageForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalPageForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal page forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.priority as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalPageForecast {
        SeasonalPageForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            amount: self.amount,
            owner_id: self.owner_id.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal page forecast in status requested.
pub fn new_seasonal_page_forecast(code: &str, label: &str) -> SeasonalPageForecast {
    SeasonalPageForecast {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        amount: 0.0,
        owner_id: "".to_string(),
        margin: 0.0,
        status: SeasonalPageForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_page_forecast("code-403", "label-617");
        assert_eq!(item.status, SeasonalPageForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_seasonal_page_forecast("code-403", "label-617");
        if item.move_to(SeasonalPageForecastStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalPageForecastStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_page_forecast("code-403", "label-617");
        item.status = SeasonalPageForecastStatus::Expired;
        assert!(
            item.move_to(SeasonalPageForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
