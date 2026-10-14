use super::*;

use acme_kit::text;

/// Where a grant review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GrantReviewStatus {
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

impl std::fmt::Display for GrantReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantReview {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub issuer: String,
    pub archived: bool,
    pub status: GrantReviewStatus,
}

impl GrantReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantReviewStatus) -> bool {
        match self.status {
            GrantReviewStatus::New => {
                match next {
                    GrantReviewStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantReviewStatus::Queued => {
                return next == GrantReviewStatus::Running;
            }
            GrantReviewStatus::Running => {
                match next {
                    GrantReviewStatus::Done | GrantReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantReviewStatus::Done => {
                match next {
                    GrantReviewStatus::Failed => {
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

    pub fn move_to(&mut self, next: GrantReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> GrantReview {
        GrantReview {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            issuer: self.issuer.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a grant review in status new.
pub fn new_grant_review(code: &str, label: &str) -> GrantReview {
    GrantReview {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        issuer: "".to_string(),
        archived: false,
        status: GrantReviewStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_grant_review("code-715", "label-856");
        assert_eq!(item.status, GrantReviewStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_grant_review("code-715", "label-856");
        if item.move_to(GrantReviewStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantReviewStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_grant_review("code-715", "label-856");
        item.status = GrantReviewStatus::Failed;
        assert!(item.move_to(GrantReviewStatus::New).is_err(), "expected move to fail");
    }
}
