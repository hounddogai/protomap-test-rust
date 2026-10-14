use super::*;

use acme_kit::text;

/// Where a page entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PageEntryStatus {
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

impl std::fmt::Display for PageEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A page entry as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageEntry {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub weight: f64,
    pub unit: String,
    pub ends_at: i64,
    pub ttl_seconds: i64,
    pub status: PageEntryStatus,
}

impl PageEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageEntryStatus) -> bool {
        match self.status {
            PageEntryStatus::Open => {
                return next == PageEntryStatus::Reopened;
            }
            PageEntryStatus::InReview => {
                return next == PageEntryStatus::Archived;
            }
            PageEntryStatus::Resolved => {
                return next == PageEntryStatus::Archived;
            }
            PageEntryStatus::Reopened => {
                match next {
                    PageEntryStatus::Archived => {
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

    pub fn move_to(&mut self, next: PageEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page entry {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageEntry {
        PageEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            weight: self.weight,
            unit: self.unit.clone(),
            ends_at: self.ends_at,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a page entry in status open.
pub fn new_page_entry(code: &str, name: &str) -> PageEntry {
    PageEntry {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        weight: 0.0,
        unit: "".to_string(),
        ends_at: 0,
        ttl_seconds: 0,
        status: PageEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_page_entry("code-884", "name-734");
        assert_eq!(item.status, PageEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_page_entry("code-884", "name-734");
        if item.move_to(PageEntryStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageEntryStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_page_entry("code-884", "name-734");
        item.status = PageEntryStatus::Archived;
        assert!(item.move_to(PageEntryStatus::Open).is_err(), "expected move to fail");
    }
}
