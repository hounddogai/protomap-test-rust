use super::*;

use acme_kit::text;

/// Where a pending page draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPageDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingPageDraftStatus {
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

impl std::fmt::Display for PendingPageDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending page draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageDraft {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub cost: f64,
    pub ratio: f64,
    pub expires_at: i64,
    pub status: PendingPageDraftStatus,
}

impl PendingPageDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPageDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPageDraftStatus) -> bool {
        match self.status {
            PendingPageDraftStatus::Planned => {
                return next == PendingPageDraftStatus::Scheduled || next == PendingPageDraftStatus::Cancelled;
            }
            PendingPageDraftStatus::Scheduled => {
                match next {
                    PendingPageDraftStatus::InProgress | PendingPageDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPageDraftStatus::InProgress => {
                match next {
                    PendingPageDraftStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPageDraftStatus::Completed => {
                return next == PendingPageDraftStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingPageDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending page draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.starts_at as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPageDraft {
        PendingPageDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            cost: self.cost,
            ratio: self.ratio,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending page draft in status planned.
pub fn new_pending_page_draft(code: &str, title: &str) -> PendingPageDraft {
    PendingPageDraft {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        cost: 0.0,
        ratio: 0.0,
        expires_at: 0,
        status: PendingPageDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_page_draft("code-774", "title-13");
        assert_eq!(item.status, PendingPageDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_pending_page_draft("code-774", "title-13");
        if item.move_to(PendingPageDraftStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPageDraftStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_page_draft("code-774", "title-13");
        item.status = PendingPageDraftStatus::Cancelled;
        assert!(
            item.move_to(PendingPageDraftStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
