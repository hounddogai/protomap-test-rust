use super::*;

use acme_kit::text;

/// Where a manual timeline forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTimelineForecastStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualTimelineForecastStatus {
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

impl std::fmt::Display for ManualTimelineForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual timeline forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTimelineForecast {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub ends_at: i64,
    pub summary: String,
    pub locale: String,
    pub floor_value: f64,
    pub unit: String,
    pub status: ManualTimelineForecastStatus,
}

impl ManualTimelineForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTimelineForecastStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTimelineForecastStatus) -> bool {
        match self.status {
            ManualTimelineForecastStatus::Draft => {
                return next == ManualTimelineForecastStatus::Pending || next == ManualTimelineForecastStatus::Active;
            }
            ManualTimelineForecastStatus::Pending => {
                return next == ManualTimelineForecastStatus::Active || next == ManualTimelineForecastStatus::Closed;
            }
            ManualTimelineForecastStatus::Active => {
                return next == ManualTimelineForecastStatus::Suspended || next == ManualTimelineForecastStatus::Closed;
            }
            ManualTimelineForecastStatus::Suspended => {
                match next {
                    ManualTimelineForecastStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualTimelineForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual timeline forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.burst as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTimelineForecast {
        ManualTimelineForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            ends_at: self.ends_at,
            summary: self.summary.clone(),
            locale: self.locale.clone(),
            floor_value: self.floor_value,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a manual timeline forecast in status draft.
pub fn new_manual_timeline_forecast(code: &str, title: &str) -> ManualTimelineForecast {
    ManualTimelineForecast {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        ends_at: 0,
        summary: "".to_string(),
        locale: "".to_string(),
        floor_value: 0.0,
        unit: "".to_string(),
        status: ManualTimelineForecastStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_timeline_forecast("code-488", "title-933");
        assert_eq!(item.status, ManualTimelineForecastStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_manual_timeline_forecast("code-488", "title-933");
        if item.move_to(ManualTimelineForecastStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTimelineForecastStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_timeline_forecast("code-488", "title-933");
        item.status = ManualTimelineForecastStatus::Closed;
        assert!(
            item.move_to(ManualTimelineForecastStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
