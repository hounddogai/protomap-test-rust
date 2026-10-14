use super::*;

use acme_kit::text;

/// Where a credential record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialRecordStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CredentialRecordStatus {
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

impl std::fmt::Display for CredentialRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One credential record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialRecord {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub external_ref: String,
    pub starts_at: i64,
    pub status: CredentialRecordStatus,
}

impl CredentialRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialRecordStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialRecordStatus) -> bool {
        match self.status {
            CredentialRecordStatus::Open => {
                return next == CredentialRecordStatus::InReview;
            }
            CredentialRecordStatus::InReview => {
                return next == CredentialRecordStatus::Reopened || next == CredentialRecordStatus::Archived;
            }
            CredentialRecordStatus::Resolved => {
                return next == CredentialRecordStatus::Archived;
            }
            CredentialRecordStatus::Reopened => {
                match next {
                    CredentialRecordStatus::Archived => {
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

    pub fn move_to(&mut self, next: CredentialRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential record {} cannot move from {} to {}",
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
        format!("{} {}", "credential record", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.starts_at as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialRecord {
        CredentialRecord {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            external_ref: self.external_ref.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a credential record in status open.
pub fn new_credential_record(code: &str, title: &str) -> CredentialRecord {
    CredentialRecord {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        external_ref: "".to_string(),
        starts_at: 0,
        status: CredentialRecordStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_credential_record("code-978", "title-447");
        assert_eq!(item.status, CredentialRecordStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_credential_record("code-978", "title-447");
        if item.move_to(CredentialRecordStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialRecordStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_credential_record("code-978", "title-447");
        item.status = CredentialRecordStatus::Archived;
        assert!(
            item.move_to(CredentialRecordStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
