use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a page summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PageSummaryStatus {
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

impl std::fmt::Display for PageSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PageSummary {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub locale: String,
    pub attributes: BTreeMap<String, String>,
    pub archived: bool,
    pub sequence: i64,
    pub quota_units: i64,
    pub status: PageSummaryStatus,
}

impl PageSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageSummaryStatus) -> bool {
        match self.status {
            PageSummaryStatus::Open => {
                match next {
                    PageSummaryStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageSummaryStatus::InReview => {
                return next == PageSummaryStatus::Reopened;
            }
            PageSummaryStatus::Resolved => {
                return next == PageSummaryStatus::Reopened || next == PageSummaryStatus::Archived;
            }
            PageSummaryStatus::Reopened => {
                match next {
                    PageSummaryStatus::Archived => {
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

    pub fn move_to(&mut self, next: PageSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.quota_units as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageSummary {
        PageSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            locale: self.locale.clone(),
            attributes: self.attributes.clone(),
            archived: self.archived,
            sequence: self.sequence,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a page summary in status open.
pub fn new_page_summary(code: &str, name: &str) -> PageSummary {
    PageSummary {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        locale: "".to_string(),
        attributes: BTreeMap::new(),
        archived: false,
        sequence: 0,
        quota_units: 0,
        status: PageSummaryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_page_summary("code-659", "name-461");
        assert_eq!(item.status, PageSummaryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_page_summary("code-659", "name-461");
        if item.move_to(PageSummaryStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageSummaryStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_page_summary("code-659", "name-461");
        item.status = PageSummaryStatus::Archived;
        assert!(item.move_to(PageSummaryStatus::Open).is_err(), "expected move to fail");
    }
}
