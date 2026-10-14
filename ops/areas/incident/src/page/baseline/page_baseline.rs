use super::*;

use acme_kit::text;

/// Where a page baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PageBaselineStatus {
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

impl std::fmt::Display for PageBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageBaseline {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub capacity: i64,
    pub weight: f64,
    pub batch_size: i64,
    pub approved: bool,
    pub ttl_seconds: i64,
    pub pinned: bool,
    pub maximum: f64,
    pub status: PageBaselineStatus,
}

impl PageBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageBaselineStatus) -> bool {
        match self.status {
            PageBaselineStatus::Open => {
                match next {
                    PageBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageBaselineStatus::InReview => {
                match next {
                    PageBaselineStatus::Resolved | PageBaselineStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageBaselineStatus::Resolved => {
                return next == PageBaselineStatus::Reopened || next == PageBaselineStatus::Archived;
            }
            PageBaselineStatus::Reopened => {
                return next == PageBaselineStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageBaseline {
        PageBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            capacity: self.capacity,
            weight: self.weight,
            batch_size: self.batch_size,
            approved: self.approved,
            ttl_seconds: self.ttl_seconds,
            pinned: self.pinned,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a page baseline in status open.
pub fn new_page_baseline(code: &str, title: &str) -> PageBaseline {
    PageBaseline {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        capacity: 0,
        weight: 0.0,
        batch_size: 0,
        approved: false,
        ttl_seconds: 0,
        pinned: false,
        maximum: 0.0,
        status: PageBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_page_baseline("code-246", "title-639");
        assert_eq!(item.status, PageBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_page_baseline("code-246", "title-639");
        if item.move_to(PageBaselineStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageBaselineStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_page_baseline("code-246", "title-639");
        item.status = PageBaselineStatus::Archived;
        assert!(item.move_to(PageBaselineStatus::Open).is_err(), "expected move to fail");
    }
}
