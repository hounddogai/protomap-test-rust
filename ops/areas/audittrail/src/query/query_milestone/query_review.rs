use super::*;

use acme_kit::text;

/// Where a query review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QueryReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QueryReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryReview {
    pub code: String,
    pub title: String,
    pub category: String,
    pub visible: bool,
    pub fingerprint: String,
    pub batch_size: i64,
    pub rotation_days: i64,
    pub status: QueryReviewStatus,
}

impl QueryReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryReviewStatus) -> bool {
        match self.status {
            QueryReviewStatus::Proposed => {
                return next == QueryReviewStatus::Withdrawn;
            }
            QueryReviewStatus::Approved => {
                return next == QueryReviewStatus::Rejected || next == QueryReviewStatus::Withdrawn;
            }
            QueryReviewStatus::Rejected => {
                match next {
                    QueryReviewStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: QueryReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "query review", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.rotation_days as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryReview {
        QueryReview {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            visible: self.visible,
            fingerprint: self.fingerprint.clone(),
            batch_size: self.batch_size,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a query review in status proposed.
pub fn new_query_review(code: &str, title: &str) -> QueryReview {
    QueryReview {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        visible: false,
        fingerprint: "".to_string(),
        batch_size: 0,
        rotation_days: 0,
        status: QueryReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_query_review("code-583", "title-45");
        assert_eq!(item.status, QueryReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_query_review("code-583", "title-45");
        if item.move_to(QueryReviewStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryReviewStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_query_review("code-583", "title-45");
        item.status = QueryReviewStatus::Withdrawn;
        assert!(
            item.move_to(QueryReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
