use super::*;

use acme_kit::text;

/// Where a pending quote group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingQuoteGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingQuoteGroupStatus {
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

impl std::fmt::Display for PendingQuoteGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending quote group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingQuoteGroup {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub starts_at: i64,
    pub key_id: String,
    pub quantity: i64,
    pub burst: i64,
    pub quota_units: i64,
    pub status: PendingQuoteGroupStatus,
}

impl PendingQuoteGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingQuoteGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingQuoteGroupStatus) -> bool {
        match self.status {
            PendingQuoteGroupStatus::Open => {
                return next == PendingQuoteGroupStatus::Reopened;
            }
            PendingQuoteGroupStatus::InReview => {
                return next == PendingQuoteGroupStatus::Resolved || next == PendingQuoteGroupStatus::Archived;
            }
            PendingQuoteGroupStatus::Resolved => {
                match next {
                    PendingQuoteGroupStatus::Reopened | PendingQuoteGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingQuoteGroupStatus::Reopened => {
                match next {
                    PendingQuoteGroupStatus::Archived => {
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

    pub fn move_to(&mut self, next: PendingQuoteGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending quote group {} cannot move from {} to {}",
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
        format!("{} {}", "pending quote group", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.starts_at as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingQuoteGroup {
        PendingQuoteGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            starts_at: self.starts_at,
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            burst: self.burst,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending quote group in status open.
pub fn new_pending_quote_group(code: &str, title: &str) -> PendingQuoteGroup {
    PendingQuoteGroup {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        starts_at: 0,
        key_id: "".to_string(),
        quantity: 0,
        burst: 0,
        quota_units: 0,
        status: PendingQuoteGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_quote_group("code-440", "title-534");
        assert_eq!(item.status, PendingQuoteGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_pending_quote_group("code-440", "title-534");
        if item.move_to(PendingQuoteGroupStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingQuoteGroupStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_quote_group("code-440", "title-534");
        item.status = PendingQuoteGroupStatus::Archived;
        assert!(
            item.move_to(PendingQuoteGroupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
