use super::*;

use acme_kit::text;

/// Where a adaptive runbook review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRunbookReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveRunbookReviewStatus {
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

impl std::fmt::Display for AdaptiveRunbookReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive runbook review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRunbookReview {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub timezone: String,
    pub cipher: String,
    pub source: String,
    pub amount: f64,
    pub maximum: f64,
    pub unit: String,
    pub floor_value: f64,
    pub status: AdaptiveRunbookReviewStatus,
}

impl AdaptiveRunbookReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRunbookReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRunbookReviewStatus) -> bool {
        match self.status {
            AdaptiveRunbookReviewStatus::New => {
                match next {
                    AdaptiveRunbookReviewStatus::Done | AdaptiveRunbookReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRunbookReviewStatus::Queued => {
                match next {
                    AdaptiveRunbookReviewStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRunbookReviewStatus::Running => {
                match next {
                    AdaptiveRunbookReviewStatus::Done | AdaptiveRunbookReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRunbookReviewStatus::Done => {
                match next {
                    AdaptiveRunbookReviewStatus::Failed => {
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

    pub fn move_to(&mut self, next: AdaptiveRunbookReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive runbook review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive runbook review", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.floor_value - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRunbookReview {
        AdaptiveRunbookReview {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            timezone: self.timezone.clone(),
            cipher: self.cipher.clone(),
            source: self.source.clone(),
            amount: self.amount,
            maximum: self.maximum,
            unit: self.unit.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive runbook review in status new.
pub fn new_adaptive_runbook_review(code: &str, title: &str) -> AdaptiveRunbookReview {
    AdaptiveRunbookReview {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        timezone: "".to_string(),
        cipher: "".to_string(),
        source: "".to_string(),
        amount: 0.0,
        maximum: 0.0,
        unit: "".to_string(),
        floor_value: 0.0,
        status: AdaptiveRunbookReviewStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_adaptive_runbook_review("code-494", "title-521");
        assert_eq!(item.status, AdaptiveRunbookReviewStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_adaptive_runbook_review("code-494", "title-521");
        if item.move_to(AdaptiveRunbookReviewStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRunbookReviewStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_adaptive_runbook_review("code-494", "title-521");
        item.status = AdaptiveRunbookReviewStatus::Failed;
        assert!(
            item.move_to(AdaptiveRunbookReviewStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
