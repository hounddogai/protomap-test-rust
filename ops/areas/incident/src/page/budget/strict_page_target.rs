use super::*;

use acme_kit::text;

/// Where a strict page target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPageTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictPageTargetStatus {
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

impl std::fmt::Display for StrictPageTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict page target as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPageTarget {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub rate: f64,
    pub timezone: String,
    pub category: String,
    pub limit: i64,
    pub status: StrictPageTargetStatus,
}

impl StrictPageTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPageTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPageTargetStatus) -> bool {
        match self.status {
            StrictPageTargetStatus::Open => {
                return next == StrictPageTargetStatus::InReview;
            }
            StrictPageTargetStatus::InReview => {
                return next == StrictPageTargetStatus::Resolved || next == StrictPageTargetStatus::Reopened;
            }
            StrictPageTargetStatus::Resolved => {
                match next {
                    StrictPageTargetStatus::Reopened | StrictPageTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPageTargetStatus::Reopened => {
                return next == StrictPageTargetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictPageTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict page target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.limit as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPageTarget {
        StrictPageTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            rate: self.rate,
            timezone: self.timezone.clone(),
            category: self.category.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict page target in status open.
pub fn new_strict_page_target(code: &str, title: &str) -> StrictPageTarget {
    StrictPageTarget {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        rate: 0.0,
        timezone: "".to_string(),
        category: "".to_string(),
        limit: 0,
        status: StrictPageTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_page_target("code-710", "title-723");
        assert_eq!(item.status, StrictPageTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_strict_page_target("code-710", "title-723");
        if item.move_to(StrictPageTargetStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictPageTargetStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_page_target("code-710", "title-723");
        item.status = StrictPageTargetStatus::Archived;
        assert!(
            item.move_to(StrictPageTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
