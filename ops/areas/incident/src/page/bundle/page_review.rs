use super::*;

use acme_kit::text;

/// Where a page review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PageReviewStatus {
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

impl std::fmt::Display for PageReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageReview {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub minimum: f64,
    pub attempts: i64,
    pub region: String,
    pub capacity: i64,
    pub status: PageReviewStatus,
}

impl PageReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageReviewStatus) -> bool {
        match self.status {
            PageReviewStatus::Open => {
                return next == PageReviewStatus::Resolved || next == PageReviewStatus::Reopened;
            }
            PageReviewStatus::InReview => {
                match next {
                    PageReviewStatus::Reopened | PageReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageReviewStatus::Resolved => {
                return next == PageReviewStatus::Reopened;
            }
            PageReviewStatus::Reopened => {
                match next {
                    PageReviewStatus::Archived => {
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

    pub fn move_to(&mut self, next: PageReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page review {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "page review", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.ends_at as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageReview {
        PageReview {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            minimum: self.minimum,
            attempts: self.attempts,
            region: self.region.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a page review in status open.
pub fn new_page_review(code: &str, name: &str) -> PageReview {
    PageReview {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        minimum: 0.0,
        attempts: 0,
        region: "".to_string(),
        capacity: 0,
        status: PageReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_page_review("code-678", "name-154");
        assert_eq!(item.status, PageReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_page_review("code-678", "name-154");
        if item.move_to(PageReviewStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageReviewStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_page_review("code-678", "name-154");
        item.status = PageReviewStatus::Archived;
        assert!(item.move_to(PageReviewStatus::Open).is_err(), "expected move to fail");
    }
}
