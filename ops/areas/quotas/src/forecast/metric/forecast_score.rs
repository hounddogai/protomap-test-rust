use super::*;

use acme_kit::text;

/// Where a forecast score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ForecastScoreStatus {
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

impl std::fmt::Display for ForecastScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast score as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastScore {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub rate: f64,
    pub source: String,
    pub floor_value: f64,
    pub locale: String,
    pub ceiling: f64,
    pub priority: i64,
    pub host: String,
    pub status: ForecastScoreStatus,
}

impl ForecastScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastScoreStatus) -> bool {
        match self.status {
            ForecastScoreStatus::Open => {
                match next {
                    ForecastScoreStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastScoreStatus::InReview => {
                return next == ForecastScoreStatus::Resolved || next == ForecastScoreStatus::Archived;
            }
            ForecastScoreStatus::Resolved => {
                return next == ForecastScoreStatus::Reopened;
            }
            ForecastScoreStatus::Reopened => {
                return next == ForecastScoreStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rate + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastScore {
        ForecastScore {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            rate: self.rate,
            source: self.source.clone(),
            floor_value: self.floor_value,
            locale: self.locale.clone(),
            ceiling: self.ceiling,
            priority: self.priority,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a forecast score in status open.
pub fn new_forecast_score(code: &str, label: &str) -> ForecastScore {
    ForecastScore {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        rate: 0.0,
        source: "".to_string(),
        floor_value: 0.0,
        locale: "".to_string(),
        ceiling: 0.0,
        priority: 0,
        host: "".to_string(),
        status: ForecastScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_forecast_score("code-156", "label-199");
        assert_eq!(item.status, ForecastScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_forecast_score("code-156", "label-199");
        if item.move_to(ForecastScoreStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastScoreStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_forecast_score("code-156", "label-199");
        item.status = ForecastScoreStatus::Archived;
        assert!(
            item.move_to(ForecastScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
