use super::*;

use acme_kit::text;

/// Where a allowlist forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AllowlistForecastStatus {
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

impl std::fmt::Display for AllowlistForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistForecast {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub scope: String,
    pub principal: String,
    pub score: f64,
    pub threshold: f64,
    pub attempts: i64,
    pub cost: f64,
    pub ceiling: f64,
    pub region: String,
    pub status: AllowlistForecastStatus,
}

impl AllowlistForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistForecastStatus) -> bool {
        match self.status {
            AllowlistForecastStatus::New => {
                return next == AllowlistForecastStatus::Done;
            }
            AllowlistForecastStatus::Queued => {
                return next == AllowlistForecastStatus::Running;
            }
            AllowlistForecastStatus::Running => {
                return next == AllowlistForecastStatus::Done || next == AllowlistForecastStatus::Failed;
            }
            AllowlistForecastStatus::Done => {
                return next == AllowlistForecastStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "allowlist forecast", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.score - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistForecast {
        AllowlistForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            scope: self.scope.clone(),
            principal: self.principal.clone(),
            score: self.score,
            threshold: self.threshold,
            attempts: self.attempts,
            cost: self.cost,
            ceiling: self.ceiling,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a allowlist forecast in status new.
pub fn new_allowlist_forecast(code: &str, title: &str) -> AllowlistForecast {
    AllowlistForecast {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        scope: "".to_string(),
        principal: "".to_string(),
        score: 0.0,
        threshold: 0.0,
        attempts: 0,
        cost: 0.0,
        ceiling: 0.0,
        region: "".to_string(),
        status: AllowlistForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_allowlist_forecast("code-874", "title-967");
        assert_eq!(item.status, AllowlistForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_allowlist_forecast("code-874", "title-967");
        if item.move_to(AllowlistForecastStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistForecastStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_allowlist_forecast("code-874", "title-967");
        item.status = AllowlistForecastStatus::Failed;
        assert!(
            item.move_to(AllowlistForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
