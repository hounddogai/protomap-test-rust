use super::*;

use acme_kit::text;

/// Where a lenient credential draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientCredentialDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientCredentialDraftStatus {
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

impl std::fmt::Display for LenientCredentialDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient credential draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCredentialDraft {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub rotation_days: i64,
    pub ends_at: i64,
    pub fingerprint: String,
    pub name: String,
    pub segment: String,
    pub tenant_id: String,
    pub quota_units: i64,
    pub sequence: i64,
    pub status: LenientCredentialDraftStatus,
}

impl LenientCredentialDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientCredentialDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientCredentialDraftStatus) -> bool {
        match self.status {
            LenientCredentialDraftStatus::Open => {
                return next == LenientCredentialDraftStatus::Reopened;
            }
            LenientCredentialDraftStatus::InReview => {
                return next == LenientCredentialDraftStatus::Reopened;
            }
            LenientCredentialDraftStatus::Resolved => {
                match next {
                    LenientCredentialDraftStatus::Reopened | LenientCredentialDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientCredentialDraftStatus::Reopened => {
                match next {
                    LenientCredentialDraftStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientCredentialDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient credential draft {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.rotation_days as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientCredentialDraft {
        LenientCredentialDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            rotation_days: self.rotation_days,
            ends_at: self.ends_at,
            fingerprint: self.fingerprint.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            tenant_id: self.tenant_id.clone(),
            quota_units: self.quota_units,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a lenient credential draft in status open.
pub fn new_lenient_credential_draft(code: &str, label: &str) -> LenientCredentialDraft {
    LenientCredentialDraft {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        rotation_days: 0,
        ends_at: 0,
        fingerprint: "".to_string(),
        name: "".to_string(),
        segment: "".to_string(),
        tenant_id: "".to_string(),
        quota_units: 0,
        sequence: 0,
        status: LenientCredentialDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_credential_draft("code-691", "label-660");
        assert_eq!(item.status, LenientCredentialDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_lenient_credential_draft("code-691", "label-660");
        if item.move_to(LenientCredentialDraftStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientCredentialDraftStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_credential_draft("code-691", "label-660");
        item.status = LenientCredentialDraftStatus::Archived;
        assert!(
            item.move_to(LenientCredentialDraftStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
