use super::*;

use acme_kit::text;

/// Where a lenient forecast estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientForecastEstimateStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientForecastEstimateStatus {
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

impl std::fmt::Display for LenientForecastEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient forecast estimate as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastEstimate {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub floor_value: f64,
    pub scope: String,
    pub title: String,
    pub channel: String,
    pub cost: f64,
    pub grace_days: i64,
    pub pinned: bool,
    pub margin: f64,
    pub status: LenientForecastEstimateStatus,
}

impl LenientForecastEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientForecastEstimateStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientForecastEstimateStatus) -> bool {
        match self.status {
            LenientForecastEstimateStatus::Planned => {
                match next {
                    LenientForecastEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientForecastEstimateStatus::Scheduled => {
                match next {
                    LenientForecastEstimateStatus::InProgress | LenientForecastEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientForecastEstimateStatus::InProgress => {
                match next {
                    LenientForecastEstimateStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientForecastEstimateStatus::Completed => {
                match next {
                    LenientForecastEstimateStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientForecastEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient forecast estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "lenient forecast estimate", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.grace_days as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientForecastEstimate {
        LenientForecastEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            floor_value: self.floor_value,
            scope: self.scope.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            cost: self.cost,
            grace_days: self.grace_days,
            pinned: self.pinned,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient forecast estimate in status planned.
pub fn new_lenient_forecast_estimate(code: &str, label: &str) -> LenientForecastEstimate {
    LenientForecastEstimate {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        floor_value: 0.0,
        scope: "".to_string(),
        title: "".to_string(),
        channel: "".to_string(),
        cost: 0.0,
        grace_days: 0,
        pinned: false,
        margin: 0.0,
        status: LenientForecastEstimateStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_forecast_estimate("code-998", "label-329");
        assert_eq!(item.status, LenientForecastEstimateStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_lenient_forecast_estimate("code-998", "label-329");
        if item.move_to(LenientForecastEstimateStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientForecastEstimateStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_forecast_estimate("code-998", "label-329");
        item.status = LenientForecastEstimateStatus::Cancelled;
        assert!(
            item.move_to(LenientForecastEstimateStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
