use super::*;

use acme_kit::text;

/// Where a runbook review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RunbookReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RunbookReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReview {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub timezone: String,
    pub retries: i64,
    pub status: RunbookReviewStatus,
}

impl RunbookReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookReviewStatus) -> bool {
        match self.status {
            RunbookReviewStatus::Created => {
                return next == RunbookReviewStatus::Published;
            }
            RunbookReviewStatus::Validated => {
                match next {
                    RunbookReviewStatus::Published | RunbookReviewStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookReviewStatus::Published => {
                return next == RunbookReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        (self.discount * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookReview {
        RunbookReview {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            timezone: self.timezone.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a runbook review in status created.
pub fn new_runbook_review(code: &str, label: &str) -> RunbookReview {
    RunbookReview {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        timezone: "".to_string(),
        retries: 0,
        status: RunbookReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_runbook_review("code-452", "label-134");
        assert_eq!(item.status, RunbookReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_runbook_review("code-452", "label-134");
        if item.move_to(RunbookReviewStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookReviewStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_runbook_review("code-452", "label-134");
        item.status = RunbookReviewStatus::Retired;
        assert!(
            item.move_to(RunbookReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
