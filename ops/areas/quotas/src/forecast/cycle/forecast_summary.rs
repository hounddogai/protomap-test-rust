use super::*;

use acme_kit::text;

/// Where a forecast summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastSummaryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ForecastSummaryStatus {
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

impl std::fmt::Display for ForecastSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSummary {
    pub code: String,
    pub title: String,
    pub country: String,
    pub minimum: f64,
    pub segment: String,
    pub port: i64,
    pub grace_days: i64,
    pub status: ForecastSummaryStatus,
}

impl ForecastSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastSummaryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastSummaryStatus) -> bool {
        match self.status {
            ForecastSummaryStatus::New => {
                match next {
                    ForecastSummaryStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastSummaryStatus::Queued => {
                match next {
                    ForecastSummaryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastSummaryStatus::Running => {
                return next == ForecastSummaryStatus::Done || next == ForecastSummaryStatus::Failed;
            }
            ForecastSummaryStatus::Done => {
                return next == ForecastSummaryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "forecast summary", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.minimum + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastSummary {
        ForecastSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            minimum: self.minimum,
            segment: self.segment.clone(),
            port: self.port,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a forecast summary in status new.
pub fn new_forecast_summary(code: &str, title: &str) -> ForecastSummary {
    ForecastSummary {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        minimum: 0.0,
        segment: "".to_string(),
        port: 0,
        grace_days: 0,
        status: ForecastSummaryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_forecast_summary("code-604", "title-886");
        assert_eq!(item.status, ForecastSummaryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_forecast_summary("code-604", "title-886");
        if item.move_to(ForecastSummaryStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastSummaryStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_forecast_summary("code-604", "title-886");
        item.status = ForecastSummaryStatus::Failed;
        assert!(
            item.move_to(ForecastSummaryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
