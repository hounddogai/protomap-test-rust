use super::*;

use acme_kit::text;

/// Where a strict forecast review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictForecastReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictForecastReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictForecastReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict forecast review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastReview {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub capacity: i64,
    pub updated_at: i64,
    pub pinned: bool,
    pub category: String,
    pub attempts: i64,
    pub port: i64,
    pub created_at: i64,
    pub status: StrictForecastReviewStatus,
}

impl StrictForecastReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictForecastReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictForecastReviewStatus) -> bool {
        match self.status {
            StrictForecastReviewStatus::Created => {
                return next == StrictForecastReviewStatus::Validated || next == StrictForecastReviewStatus::Published;
            }
            StrictForecastReviewStatus::Validated => {
                match next {
                    StrictForecastReviewStatus::Published | StrictForecastReviewStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictForecastReviewStatus::Published => {
                return next == StrictForecastReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictForecastReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict forecast review {} cannot move from {} to {}",
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

    pub fn effective_volume(&self) -> f64 {
        ((self.port as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictForecastReview {
        StrictForecastReview {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            capacity: self.capacity,
            updated_at: self.updated_at,
            pinned: self.pinned,
            category: self.category.clone(),
            attempts: self.attempts,
            port: self.port,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict forecast review in status created.
pub fn new_strict_forecast_review(code: &str, title: &str) -> StrictForecastReview {
    StrictForecastReview {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        capacity: 0,
        updated_at: 0,
        pinned: false,
        category: "".to_string(),
        attempts: 0,
        port: 0,
        created_at: 0,
        status: StrictForecastReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_forecast_review("code-834", "title-501");
        assert_eq!(item.status, StrictForecastReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_strict_forecast_review("code-834", "title-501");
        if item.move_to(StrictForecastReviewStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictForecastReviewStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_forecast_review("code-834", "title-501");
        item.status = StrictForecastReviewStatus::Retired;
        assert!(
            item.move_to(StrictForecastReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
