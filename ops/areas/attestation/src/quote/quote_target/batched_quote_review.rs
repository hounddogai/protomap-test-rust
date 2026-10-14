use super::*;

use acme_kit::text;

/// Where a batched quote review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedQuoteReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedQuoteReviewStatus {
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

impl std::fmt::Display for BatchedQuoteReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched quote review as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteReview {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub visible: bool,
    pub rotation_days: i64,
    pub currency: String,
    pub country: String,
    pub unit: String,
    pub source: String,
    pub status: BatchedQuoteReviewStatus,
}

impl BatchedQuoteReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedQuoteReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedQuoteReviewStatus) -> bool {
        match self.status {
            BatchedQuoteReviewStatus::New => {
                return next == BatchedQuoteReviewStatus::Queued;
            }
            BatchedQuoteReviewStatus::Queued => {
                match next {
                    BatchedQuoteReviewStatus::Running | BatchedQuoteReviewStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQuoteReviewStatus::Running => {
                match next {
                    BatchedQuoteReviewStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQuoteReviewStatus::Done => {
                return next == BatchedQuoteReviewStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedQuoteReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched quote review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched quote review", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> BatchedQuoteReview {
        BatchedQuoteReview {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            visible: self.visible,
            rotation_days: self.rotation_days,
            currency: self.currency.clone(),
            country: self.country.clone(),
            unit: self.unit.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a batched quote review in status new.
pub fn new_batched_quote_review(code: &str, label: &str) -> BatchedQuoteReview {
    BatchedQuoteReview {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        visible: false,
        rotation_days: 0,
        currency: "".to_string(),
        country: "".to_string(),
        unit: "".to_string(),
        source: "".to_string(),
        status: BatchedQuoteReviewStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_quote_review("code-575", "label-966");
        assert_eq!(item.status, BatchedQuoteReviewStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_batched_quote_review("code-575", "label-966");
        if item.move_to(BatchedQuoteReviewStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedQuoteReviewStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_quote_review("code-575", "label-966");
        item.status = BatchedQuoteReviewStatus::Failed;
        assert!(
            item.move_to(BatchedQuoteReviewStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
