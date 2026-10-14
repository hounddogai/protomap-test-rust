use super::*;

use acme_kit::text;

/// Where a primary isolation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryIsolationForecastStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryIsolationForecastStatus {
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

impl std::fmt::Display for PrimaryIsolationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary isolation forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIsolationForecast {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub fingerprint: String,
    pub attempts: i64,
    pub batch_size: i64,
    pub status: PrimaryIsolationForecastStatus,
}

impl PrimaryIsolationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryIsolationForecastStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryIsolationForecastStatus) -> bool {
        match self.status {
            PrimaryIsolationForecastStatus::Open => {
                return next == PrimaryIsolationForecastStatus::Resolved
                    || next == PrimaryIsolationForecastStatus::Archived;
            }
            PrimaryIsolationForecastStatus::InReview => {
                match next {
                    PrimaryIsolationForecastStatus::Resolved | PrimaryIsolationForecastStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryIsolationForecastStatus::Resolved => {
                return next == PrimaryIsolationForecastStatus::Reopened;
            }
            PrimaryIsolationForecastStatus::Reopened => {
                return next == PrimaryIsolationForecastStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryIsolationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary isolation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.attempts as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryIsolationForecast {
        PrimaryIsolationForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            fingerprint: self.fingerprint.clone(),
            attempts: self.attempts,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary isolation forecast in status open.
pub fn new_primary_isolation_forecast(code: &str, title: &str) -> PrimaryIsolationForecast {
    PrimaryIsolationForecast {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        fingerprint: "".to_string(),
        attempts: 0,
        batch_size: 0,
        status: PrimaryIsolationForecastStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_isolation_forecast("code-913", "title-1");
        assert_eq!(item.status, PrimaryIsolationForecastStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_primary_isolation_forecast("code-913", "title-1");
        if item.move_to(PrimaryIsolationForecastStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryIsolationForecastStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_isolation_forecast("code-913", "title-1");
        item.status = PrimaryIsolationForecastStatus::Archived;
        assert!(
            item.move_to(PrimaryIsolationForecastStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
