use super::*;

use acme_kit::text;

/// Where a page policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PagePolicyStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PagePolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PagePolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a page policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PagePolicy {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub attempts: i64,
    pub category: String,
    pub ends_at: i64,
    pub quota_units: i64,
    pub status: PagePolicyStatus,
}

impl PagePolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PagePolicyStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PagePolicyStatus) -> bool {
        match self.status {
            PagePolicyStatus::Planned => {
                return next == PagePolicyStatus::Scheduled || next == PagePolicyStatus::Completed;
            }
            PagePolicyStatus::Scheduled => {
                match next {
                    PagePolicyStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PagePolicyStatus::InProgress => {
                match next {
                    PagePolicyStatus::Completed | PagePolicyStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PagePolicyStatus::Completed => {
                return next == PagePolicyStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PagePolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page policy {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ends_at as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PagePolicy {
        PagePolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            tags: self.tags.clone(),
            attempts: self.attempts,
            category: self.category.clone(),
            ends_at: self.ends_at,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a page policy in status planned.
pub fn new_page_policy(code: &str, label: &str) -> PagePolicy {
    PagePolicy {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        tags: Vec::new(),
        attempts: 0,
        category: "".to_string(),
        ends_at: 0,
        quota_units: 0,
        status: PagePolicyStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_page_policy("code-769", "label-922");
        assert_eq!(item.status, PagePolicyStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_page_policy("code-769", "label-922");
        if item.move_to(PagePolicyStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PagePolicyStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_page_policy("code-769", "label-922");
        item.status = PagePolicyStatus::Cancelled;
        assert!(
            item.move_to(PagePolicyStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
