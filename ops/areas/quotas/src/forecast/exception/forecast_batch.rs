use super::*;

use acme_kit::text;

/// Where a forecast batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ForecastBatchStatus {
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

impl std::fmt::Display for ForecastBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBatch {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub budget: f64,
    pub ttl_seconds: i64,
    pub fingerprint: String,
    pub status: ForecastBatchStatus,
}

impl ForecastBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastBatchStatus) -> bool {
        match self.status {
            ForecastBatchStatus::Open => {
                return next == ForecastBatchStatus::InReview;
            }
            ForecastBatchStatus::InReview => {
                return next == ForecastBatchStatus::Reopened || next == ForecastBatchStatus::Archived;
            }
            ForecastBatchStatus::Resolved => {
                match next {
                    ForecastBatchStatus::Reopened | ForecastBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastBatchStatus::Reopened => {
                match next {
                    ForecastBatchStatus::Archived => {
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

    pub fn move_to(&mut self, next: ForecastBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast batch {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "forecast batch", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastBatch {
        ForecastBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            budget: self.budget,
            ttl_seconds: self.ttl_seconds,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a forecast batch in status open.
pub fn new_forecast_batch(code: &str, label: &str) -> ForecastBatch {
    ForecastBatch {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        budget: 0.0,
        ttl_seconds: 0,
        fingerprint: "".to_string(),
        status: ForecastBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_forecast_batch("code-718", "label-577");
        assert_eq!(item.status, ForecastBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_forecast_batch("code-718", "label-577");
        if item.move_to(ForecastBatchStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastBatchStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_forecast_batch("code-718", "label-577");
        item.status = ForecastBatchStatus::Archived;
        assert!(
            item.move_to(ForecastBatchStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
