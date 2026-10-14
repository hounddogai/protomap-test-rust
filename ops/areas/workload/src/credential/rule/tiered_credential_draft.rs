use super::*;

use acme_kit::text;

/// Where a tiered credential draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredCredentialDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TieredCredentialDraftStatus {
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

impl std::fmt::Display for TieredCredentialDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered credential draft as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialDraft {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub maximum: f64,
    pub category: String,
    pub status: TieredCredentialDraftStatus,
}

impl TieredCredentialDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredCredentialDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredCredentialDraftStatus) -> bool {
        match self.status {
            TieredCredentialDraftStatus::Planned => {
                return next == TieredCredentialDraftStatus::Scheduled
                    || next == TieredCredentialDraftStatus::InProgress;
            }
            TieredCredentialDraftStatus::Scheduled => {
                match next {
                    TieredCredentialDraftStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredCredentialDraftStatus::InProgress => {
                return next == TieredCredentialDraftStatus::Cancelled;
            }
            TieredCredentialDraftStatus::Completed => {
                match next {
                    TieredCredentialDraftStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TieredCredentialDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered credential draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered credential draft", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> TieredCredentialDraft {
        TieredCredentialDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            maximum: self.maximum,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a tiered credential draft in status planned.
pub fn new_tiered_credential_draft(code: &str, title: &str) -> TieredCredentialDraft {
    TieredCredentialDraft {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        maximum: 0.0,
        category: "".to_string(),
        status: TieredCredentialDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_tiered_credential_draft("code-458", "title-279");
        assert_eq!(item.status, TieredCredentialDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_tiered_credential_draft("code-458", "title-279");
        if item.move_to(TieredCredentialDraftStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredCredentialDraftStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_tiered_credential_draft("code-458", "title-279");
        item.status = TieredCredentialDraftStatus::Cancelled;
        assert!(
            item.move_to(TieredCredentialDraftStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
