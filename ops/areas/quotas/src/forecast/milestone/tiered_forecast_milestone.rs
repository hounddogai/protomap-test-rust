use super::*;

use acme_kit::text;

/// Where a tiered forecast milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredForecastMilestoneStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredForecastMilestoneStatus {
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

impl std::fmt::Display for TieredForecastMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered forecast milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastMilestone {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub name: String,
    pub unit: String,
    pub quota_units: i64,
    pub ratio: f64,
    pub cost: f64,
    pub rotation_days: i64,
    pub threshold: f64,
    pub attempts: i64,
    pub status: TieredForecastMilestoneStatus,
}

impl TieredForecastMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredForecastMilestoneStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredForecastMilestoneStatus) -> bool {
        match self.status {
            TieredForecastMilestoneStatus::Open => {
                match next {
                    TieredForecastMilestoneStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredForecastMilestoneStatus::InReview => {
                match next {
                    TieredForecastMilestoneStatus::Reopened | TieredForecastMilestoneStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredForecastMilestoneStatus::Resolved => {
                return next == TieredForecastMilestoneStatus::Reopened;
            }
            TieredForecastMilestoneStatus::Reopened => {
                match next {
                    TieredForecastMilestoneStatus::Archived => {
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

    pub fn move_to(&mut self, next: TieredForecastMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered forecast milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.attempts as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredForecastMilestone {
        TieredForecastMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            quota_units: self.quota_units,
            ratio: self.ratio,
            cost: self.cost,
            rotation_days: self.rotation_days,
            threshold: self.threshold,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tiered forecast milestone in status open.
pub fn new_tiered_forecast_milestone(code: &str, label: &str) -> TieredForecastMilestone {
    TieredForecastMilestone {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        name: "".to_string(),
        unit: "".to_string(),
        quota_units: 0,
        ratio: 0.0,
        cost: 0.0,
        rotation_days: 0,
        threshold: 0.0,
        attempts: 0,
        status: TieredForecastMilestoneStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_forecast_milestone("code-616", "label-146");
        assert_eq!(item.status, TieredForecastMilestoneStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_tiered_forecast_milestone("code-616", "label-146");
        if item.move_to(TieredForecastMilestoneStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredForecastMilestoneStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_forecast_milestone("code-616", "label-146");
        item.status = TieredForecastMilestoneStatus::Archived;
        assert!(
            item.move_to(TieredForecastMilestoneStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
