use super::*;

use acme_kit::text;

/// Where a capped runbook request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRunbookRequestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedRunbookRequestStatus {
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

impl std::fmt::Display for CappedRunbookRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped runbook request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRunbookRequest {
    pub code: String,
    pub label: String,
    pub category: String,
    pub timezone: String,
    pub quota_units: i64,
    pub ends_at: i64,
    pub notes: String,
    pub status: CappedRunbookRequestStatus,
}

impl CappedRunbookRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRunbookRequestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRunbookRequestStatus) -> bool {
        match self.status {
            CappedRunbookRequestStatus::Open => {
                return next == CappedRunbookRequestStatus::Reopened || next == CappedRunbookRequestStatus::Archived;
            }
            CappedRunbookRequestStatus::InReview => {
                return next == CappedRunbookRequestStatus::Reopened;
            }
            CappedRunbookRequestStatus::Resolved => {
                match next {
                    CappedRunbookRequestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRunbookRequestStatus::Reopened => {
                return next == CappedRunbookRequestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRunbookRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped runbook request {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ends_at as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRunbookRequest {
        CappedRunbookRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            timezone: self.timezone.clone(),
            quota_units: self.quota_units,
            ends_at: self.ends_at,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a capped runbook request in status open.
pub fn new_capped_runbook_request(code: &str, label: &str) -> CappedRunbookRequest {
    CappedRunbookRequest {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        timezone: "".to_string(),
        quota_units: 0,
        ends_at: 0,
        notes: "".to_string(),
        status: CappedRunbookRequestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_runbook_request("code-147", "label-995");
        assert_eq!(item.status, CappedRunbookRequestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_capped_runbook_request("code-147", "label-995");
        if item.move_to(CappedRunbookRequestStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedRunbookRequestStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_runbook_request("code-147", "label-995");
        item.status = CappedRunbookRequestStatus::Archived;
        assert!(
            item.move_to(CappedRunbookRequestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
