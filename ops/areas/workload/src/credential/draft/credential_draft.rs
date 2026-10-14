use super::*;

use acme_kit::text;

/// Where a credential draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialDraftStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CredentialDraftStatus {
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

impl std::fmt::Display for CredentialDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential draft as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialDraft {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub discount: f64,
    pub score: f64,
    pub tags: Vec<String>,
    pub description: String,
    pub category: String,
    pub window_days: i64,
    pub status: CredentialDraftStatus,
}

impl CredentialDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialDraftStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialDraftStatus) -> bool {
        match self.status {
            CredentialDraftStatus::New => {
                return next == CredentialDraftStatus::Failed;
            }
            CredentialDraftStatus::Queued => {
                match next {
                    CredentialDraftStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialDraftStatus::Running => {
                match next {
                    CredentialDraftStatus::Done | CredentialDraftStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialDraftStatus::Done => {
                match next {
                    CredentialDraftStatus::Failed => {
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

    pub fn move_to(&mut self, next: CredentialDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.discount - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialDraft {
        CredentialDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            discount: self.discount,
            score: self.score,
            tags: self.tags.clone(),
            description: self.description.clone(),
            category: self.category.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a credential draft in status new.
pub fn new_credential_draft(code: &str, name: &str) -> CredentialDraft {
    CredentialDraft {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        discount: 0.0,
        score: 0.0,
        tags: Vec::new(),
        description: "".to_string(),
        category: "".to_string(),
        window_days: 0,
        status: CredentialDraftStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_credential_draft("code-242", "name-451");
        assert_eq!(item.status, CredentialDraftStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_credential_draft("code-242", "name-451");
        if item.move_to(CredentialDraftStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialDraftStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_credential_draft("code-242", "name-451");
        item.status = CredentialDraftStatus::Failed;
        assert!(
            item.move_to(CredentialDraftStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
