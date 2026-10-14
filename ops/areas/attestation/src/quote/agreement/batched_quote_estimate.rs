use super::*;

use acme_kit::text;

/// Where a batched quote estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedQuoteEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedQuoteEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedQuoteEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched quote estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteEstimate {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub retries: i64,
    pub scope: String,
    pub key_id: String,
    pub attempts: i64,
    pub source: String,
    pub timezone: String,
    pub batch_size: i64,
    pub country: String,
    pub status: BatchedQuoteEstimateStatus,
}

impl BatchedQuoteEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedQuoteEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedQuoteEstimateStatus) -> bool {
        match self.status {
            BatchedQuoteEstimateStatus::Idle => {
                return next == BatchedQuoteEstimateStatus::Warming || next == BatchedQuoteEstimateStatus::Ready;
            }
            BatchedQuoteEstimateStatus::Warming => {
                match next {
                    BatchedQuoteEstimateStatus::Ready | BatchedQuoteEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQuoteEstimateStatus::Ready => {
                return next == BatchedQuoteEstimateStatus::Draining || next == BatchedQuoteEstimateStatus::Stopped;
            }
            BatchedQuoteEstimateStatus::Draining => {
                match next {
                    BatchedQuoteEstimateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedQuoteEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched quote estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.batch_size as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedQuoteEstimate {
        BatchedQuoteEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            retries: self.retries,
            scope: self.scope.clone(),
            key_id: self.key_id.clone(),
            attempts: self.attempts,
            source: self.source.clone(),
            timezone: self.timezone.clone(),
            batch_size: self.batch_size,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched quote estimate in status idle.
pub fn new_batched_quote_estimate(code: &str, title: &str) -> BatchedQuoteEstimate {
    BatchedQuoteEstimate {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        retries: 0,
        scope: "".to_string(),
        key_id: "".to_string(),
        attempts: 0,
        source: "".to_string(),
        timezone: "".to_string(),
        batch_size: 0,
        country: "".to_string(),
        status: BatchedQuoteEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_quote_estimate("code-124", "title-74");
        assert_eq!(item.status, BatchedQuoteEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_batched_quote_estimate("code-124", "title-74");
        if item.move_to(BatchedQuoteEstimateStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedQuoteEstimateStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_quote_estimate("code-124", "title-74");
        item.status = BatchedQuoteEstimateStatus::Stopped;
        assert!(
            item.move_to(BatchedQuoteEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
