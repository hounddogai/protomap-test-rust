use super::*;

use acme_kit::text;

/// Where a lenient overage forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientOverageForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientOverageForecastStatus {
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

impl std::fmt::Display for LenientOverageForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient overage forecast as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientOverageForecast {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub score: f64,
    pub pinned: bool,
    pub grace_days: i64,
    pub scope: String,
    pub currency: String,
    pub cluster: String,
    pub weight: f64,
    pub approved: bool,
    pub status: LenientOverageForecastStatus,
}

impl LenientOverageForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientOverageForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientOverageForecastStatus) -> bool {
        match self.status {
            LenientOverageForecastStatus::New => {
                match next {
                    LenientOverageForecastStatus::Queued | LenientOverageForecastStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientOverageForecastStatus::Queued => {
                return next == LenientOverageForecastStatus::Failed;
            }
            LenientOverageForecastStatus::Running => {
                return next == LenientOverageForecastStatus::Done;
            }
            LenientOverageForecastStatus::Done => {
                return next == LenientOverageForecastStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientOverageForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient overage forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.weight + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientOverageForecast {
        LenientOverageForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            score: self.score,
            pinned: self.pinned,
            grace_days: self.grace_days,
            scope: self.scope.clone(),
            currency: self.currency.clone(),
            cluster: self.cluster.clone(),
            weight: self.weight,
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient overage forecast in status new.
pub fn new_lenient_overage_forecast(code: &str, label: &str) -> LenientOverageForecast {
    LenientOverageForecast {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        score: 0.0,
        pinned: false,
        grace_days: 0,
        scope: "".to_string(),
        currency: "".to_string(),
        cluster: "".to_string(),
        weight: 0.0,
        approved: false,
        status: LenientOverageForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_overage_forecast("code-719", "label-631");
        assert_eq!(item.status, LenientOverageForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_lenient_overage_forecast("code-719", "label-631");
        if item.move_to(LenientOverageForecastStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientOverageForecastStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_overage_forecast("code-719", "label-631");
        item.status = LenientOverageForecastStatus::Failed;
        assert!(
            item.move_to(LenientOverageForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
