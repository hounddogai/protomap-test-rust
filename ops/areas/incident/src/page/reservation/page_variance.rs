use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a page variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PageVarianceStatus {
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

impl std::fmt::Display for PageVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One page variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PageVariance {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub active: bool,
    pub pinned: bool,
    pub fingerprint: String,
    pub segment: String,
    pub summary: String,
    pub attributes: BTreeMap<String, String>,
    pub notes: String,
    pub threshold: f64,
    pub status: PageVarianceStatus,
}

impl PageVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageVarianceStatus) -> bool {
        match self.status {
            PageVarianceStatus::New => {
                return next == PageVarianceStatus::Queued || next == PageVarianceStatus::Done;
            }
            PageVarianceStatus::Queued => {
                match next {
                    PageVarianceStatus::Running | PageVarianceStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageVarianceStatus::Running => {
                return next == PageVarianceStatus::Done || next == PageVarianceStatus::Failed;
            }
            PageVarianceStatus::Done => {
                return next == PageVarianceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "page variance", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> PageVariance {
        PageVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            active: self.active,
            pinned: self.pinned,
            fingerprint: self.fingerprint.clone(),
            segment: self.segment.clone(),
            summary: self.summary.clone(),
            attributes: self.attributes.clone(),
            notes: self.notes.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a page variance in status new.
pub fn new_page_variance(code: &str, title: &str) -> PageVariance {
    PageVariance {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        active: false,
        pinned: false,
        fingerprint: "".to_string(),
        segment: "".to_string(),
        summary: "".to_string(),
        attributes: BTreeMap::new(),
        notes: "".to_string(),
        threshold: 0.0,
        status: PageVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_page_variance("code-837", "title-710");
        assert_eq!(item.status, PageVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_page_variance("code-837", "title-710");
        if item.move_to(PageVarianceStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageVarianceStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_page_variance("code-837", "title-710");
        item.status = PageVarianceStatus::Failed;
        assert!(item.move_to(PageVarianceStatus::New).is_err(), "expected move to fail");
    }
}
