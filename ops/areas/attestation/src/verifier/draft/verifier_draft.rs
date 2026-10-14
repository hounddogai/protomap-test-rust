use super::*;

use acme_kit::text;

/// Where a verifier draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl VerifierDraftStatus {
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

impl std::fmt::Display for VerifierDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier draft as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierDraft {
    pub code: String,
    pub title: String,
    pub description: String,
    pub ttl_seconds: i64,
    pub country: String,
    pub audience: String,
    pub starts_at: i64,
    pub threshold: f64,
    pub status: VerifierDraftStatus,
}

impl VerifierDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierDraftStatus) -> bool {
        match self.status {
            VerifierDraftStatus::Open => {
                match next {
                    VerifierDraftStatus::Resolved | VerifierDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierDraftStatus::InReview => {
                return next == VerifierDraftStatus::Resolved || next == VerifierDraftStatus::Archived;
            }
            VerifierDraftStatus::Resolved => {
                match next {
                    VerifierDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierDraftStatus::Reopened => {
                match next {
                    VerifierDraftStatus::Archived => {
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

    pub fn move_to(&mut self, next: VerifierDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier draft {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.starts_at as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierDraft {
        VerifierDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            ttl_seconds: self.ttl_seconds,
            country: self.country.clone(),
            audience: self.audience.clone(),
            starts_at: self.starts_at,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a verifier draft in status open.
pub fn new_verifier_draft(code: &str, title: &str) -> VerifierDraft {
    VerifierDraft {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        ttl_seconds: 0,
        country: "".to_string(),
        audience: "".to_string(),
        starts_at: 0,
        threshold: 0.0,
        status: VerifierDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_verifier_draft("code-239", "title-481");
        assert_eq!(item.status, VerifierDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_verifier_draft("code-239", "title-481");
        if item.move_to(VerifierDraftStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierDraftStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_verifier_draft("code-239", "title-481");
        item.status = VerifierDraftStatus::Archived;
        assert!(
            item.move_to(VerifierDraftStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
