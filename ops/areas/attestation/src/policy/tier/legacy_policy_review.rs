use super::*;

use acme_kit::text;

/// Where a legacy policy review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPolicyReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyPolicyReviewStatus {
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

impl std::fmt::Display for LegacyPolicyReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy policy review as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyReview {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub floor_value: f64,
    pub minimum: f64,
    pub limit: i64,
    pub sequence: i64,
    pub amount: f64,
    pub phone: String,
    pub score: f64,
    pub unit: String,
    pub status: LegacyPolicyReviewStatus,
}

impl LegacyPolicyReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPolicyReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPolicyReviewStatus) -> bool {
        match self.status {
            LegacyPolicyReviewStatus::New => {
                return next == LegacyPolicyReviewStatus::Running || next == LegacyPolicyReviewStatus::Failed;
            }
            LegacyPolicyReviewStatus::Queued => {
                return next == LegacyPolicyReviewStatus::Running || next == LegacyPolicyReviewStatus::Done;
            }
            LegacyPolicyReviewStatus::Running => {
                return next == LegacyPolicyReviewStatus::Done || next == LegacyPolicyReviewStatus::Failed;
            }
            LegacyPolicyReviewStatus::Done => {
                match next {
                    LegacyPolicyReviewStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyPolicyReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy policy review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.amount - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyPolicyReview {
        LegacyPolicyReview {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            floor_value: self.floor_value,
            minimum: self.minimum,
            limit: self.limit,
            sequence: self.sequence,
            amount: self.amount,
            phone: self.phone.clone(),
            score: self.score,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a legacy policy review in status new.
pub fn new_legacy_policy_review(code: &str, title: &str) -> LegacyPolicyReview {
    LegacyPolicyReview {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        floor_value: 0.0,
        minimum: 0.0,
        limit: 0,
        sequence: 0,
        amount: 0.0,
        phone: "".to_string(),
        score: 0.0,
        unit: "".to_string(),
        status: LegacyPolicyReviewStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_policy_review("code-835", "title-177");
        assert_eq!(item.status, LegacyPolicyReviewStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_legacy_policy_review("code-835", "title-177");
        if item.move_to(LegacyPolicyReviewStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyPolicyReviewStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_policy_review("code-835", "title-177");
        item.status = LegacyPolicyReviewStatus::Failed;
        assert!(
            item.move_to(LegacyPolicyReviewStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
