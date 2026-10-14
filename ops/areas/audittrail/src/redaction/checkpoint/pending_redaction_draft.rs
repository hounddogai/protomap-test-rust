use super::*;

use acme_kit::text;

/// Where a pending redaction draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRedactionDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingRedactionDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingRedactionDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending redaction draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionDraft {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub currency: String,
    pub active: bool,
    pub ends_at: i64,
    pub status: PendingRedactionDraftStatus,
}

impl PendingRedactionDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRedactionDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRedactionDraftStatus) -> bool {
        match self.status {
            PendingRedactionDraftStatus::Requested => {
                return next == PendingRedactionDraftStatus::Revoked;
            }
            PendingRedactionDraftStatus::Granted => {
                return next == PendingRedactionDraftStatus::Revoked;
            }
            PendingRedactionDraftStatus::Revoked => {
                return next == PendingRedactionDraftStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRedactionDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending redaction draft {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRedactionDraft {
        PendingRedactionDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            currency: self.currency.clone(),
            active: self.active,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending redaction draft in status requested.
pub fn new_pending_redaction_draft(code: &str, label: &str) -> PendingRedactionDraft {
    PendingRedactionDraft {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        currency: "".to_string(),
        active: false,
        ends_at: 0,
        status: PendingRedactionDraftStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_redaction_draft("code-472", "label-736");
        assert_eq!(item.status, PendingRedactionDraftStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_pending_redaction_draft("code-472", "label-736");
        if item.move_to(PendingRedactionDraftStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRedactionDraftStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_redaction_draft("code-472", "label-736");
        item.status = PendingRedactionDraftStatus::Expired;
        assert!(
            item.move_to(PendingRedactionDraftStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
