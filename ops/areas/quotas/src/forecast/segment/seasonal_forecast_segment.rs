use super::*;

use acme_kit::text;

/// Where a seasonal forecast segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalForecastSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalForecastSegmentStatus {
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

impl std::fmt::Display for SeasonalForecastSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal forecast segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalForecastSegment {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub issuer: String,
    pub quota_units: i64,
    pub status: SeasonalForecastSegmentStatus,
}

impl SeasonalForecastSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalForecastSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalForecastSegmentStatus) -> bool {
        match self.status {
            SeasonalForecastSegmentStatus::Open => {
                match next {
                    SeasonalForecastSegmentStatus::Resolved | SeasonalForecastSegmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalForecastSegmentStatus::InReview => {
                return next == SeasonalForecastSegmentStatus::Reopened;
            }
            SeasonalForecastSegmentStatus::Resolved => {
                match next {
                    SeasonalForecastSegmentStatus::Reopened | SeasonalForecastSegmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalForecastSegmentStatus::Reopened => {
                return next == SeasonalForecastSegmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalForecastSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal forecast segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.threshold + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalForecastSegment {
        SeasonalForecastSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            issuer: self.issuer.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal forecast segment in status open.
pub fn new_seasonal_forecast_segment(code: &str, label: &str) -> SeasonalForecastSegment {
    SeasonalForecastSegment {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        issuer: "".to_string(),
        quota_units: 0,
        status: SeasonalForecastSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_forecast_segment("code-283", "label-532");
        assert_eq!(item.status, SeasonalForecastSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_seasonal_forecast_segment("code-283", "label-532");
        if item.move_to(SeasonalForecastSegmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalForecastSegmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_forecast_segment("code-283", "label-532");
        item.status = SeasonalForecastSegmentStatus::Archived;
        assert!(
            item.move_to(SeasonalForecastSegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
