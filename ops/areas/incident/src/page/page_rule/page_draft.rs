use super::*;

use acme_kit::text;

/// Where a page draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PageDraftStatus {
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

impl std::fmt::Display for PageDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a page draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PageDraft {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub locale: String,
    pub summary: String,
    pub ttl_seconds: i64,
    pub visible: bool,
    pub issuer: String,
    pub archived: bool,
    pub grace_days: i64,
    pub updated_at: i64,
    pub status: PageDraftStatus,
}

impl PageDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageDraftStatus) -> bool {
        match self.status {
            PageDraftStatus::Planned => {
                return next == PageDraftStatus::Scheduled || next == PageDraftStatus::Cancelled;
            }
            PageDraftStatus::Scheduled => {
                return next == PageDraftStatus::InProgress || next == PageDraftStatus::Cancelled;
            }
            PageDraftStatus::InProgress => {
                match next {
                    PageDraftStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageDraftStatus::Completed => {
                return next == PageDraftStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageDraft {
        PageDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            locale: self.locale.clone(),
            summary: self.summary.clone(),
            ttl_seconds: self.ttl_seconds,
            visible: self.visible,
            issuer: self.issuer.clone(),
            archived: self.archived,
            grace_days: self.grace_days,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a page draft in status planned.
pub fn new_page_draft(code: &str, label: &str) -> PageDraft {
    PageDraft {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        locale: "".to_string(),
        summary: "".to_string(),
        ttl_seconds: 0,
        visible: false,
        issuer: "".to_string(),
        archived: false,
        grace_days: 0,
        updated_at: 0,
        status: PageDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_page_draft("code-472", "label-317");
        assert_eq!(item.status, PageDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_page_draft("code-472", "label-317");
        if item.move_to(PageDraftStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageDraftStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_page_draft("code-472", "label-317");
        item.status = PageDraftStatus::Cancelled;
        assert!(item.move_to(PageDraftStatus::Planned).is_err(), "expected move to fail");
    }
}
