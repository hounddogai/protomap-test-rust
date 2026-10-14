use super::*;

use acme_kit::text;

/// Where a batched forecast review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedForecastReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedForecastReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedForecastReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched forecast review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastReview {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub scope: String,
    pub notes: String,
    pub status: BatchedForecastReviewStatus,
}

impl BatchedForecastReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedForecastReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedForecastReviewStatus) -> bool {
        match self.status {
            BatchedForecastReviewStatus::Requested => {
                return next == BatchedForecastReviewStatus::Expired;
            }
            BatchedForecastReviewStatus::Granted => {
                match next {
                    BatchedForecastReviewStatus::Revoked | BatchedForecastReviewStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedForecastReviewStatus::Revoked => {
                return next == BatchedForecastReviewStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedForecastReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched forecast review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched forecast review", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> BatchedForecastReview {
        BatchedForecastReview {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            scope: self.scope.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched forecast review in status requested.
pub fn new_batched_forecast_review(code: &str, name: &str) -> BatchedForecastReview {
    BatchedForecastReview {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        scope: "".to_string(),
        notes: "".to_string(),
        status: BatchedForecastReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_forecast_review("code-213", "name-61");
        assert_eq!(item.status, BatchedForecastReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_batched_forecast_review("code-213", "name-61");
        if item.move_to(BatchedForecastReviewStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedForecastReviewStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_forecast_review("code-213", "name-61");
        item.status = BatchedForecastReviewStatus::Expired;
        assert!(
            item.move_to(BatchedForecastReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
