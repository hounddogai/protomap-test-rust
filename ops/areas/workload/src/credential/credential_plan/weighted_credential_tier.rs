use super::*;

use acme_kit::text;

/// Where a weighted credential tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedCredentialTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedCredentialTierStatus {
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

impl std::fmt::Display for WeightedCredentialTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted credential tier as the credential team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialTier {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub description: String,
    pub capacity: i64,
    pub source: String,
    pub archived: bool,
    pub status: WeightedCredentialTierStatus,
}

impl WeightedCredentialTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedCredentialTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedCredentialTierStatus) -> bool {
        match self.status {
            WeightedCredentialTierStatus::Open => {
                return next == WeightedCredentialTierStatus::Resolved
                    || next == WeightedCredentialTierStatus::Archived;
            }
            WeightedCredentialTierStatus::InReview => {
                match next {
                    WeightedCredentialTierStatus::Reopened | WeightedCredentialTierStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedCredentialTierStatus::Resolved => {
                return next == WeightedCredentialTierStatus::Archived;
            }
            WeightedCredentialTierStatus::Reopened => {
                match next {
                    WeightedCredentialTierStatus::Archived => {
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

    pub fn move_to(&mut self, next: WeightedCredentialTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted credential tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.capacity as f64
    }

    pub fn copy(&self) -> WeightedCredentialTier {
        WeightedCredentialTier {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            description: self.description.clone(),
            capacity: self.capacity,
            source: self.source.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a weighted credential tier in status open.
pub fn new_weighted_credential_tier(code: &str, title: &str) -> WeightedCredentialTier {
    WeightedCredentialTier {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        description: "".to_string(),
        capacity: 0,
        source: "".to_string(),
        archived: false,
        status: WeightedCredentialTierStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_credential_tier("code-78", "title-409");
        assert_eq!(item.status, WeightedCredentialTierStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_weighted_credential_tier("code-78", "title-409");
        if item.move_to(WeightedCredentialTierStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedCredentialTierStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_credential_tier("code-78", "title-409");
        item.status = WeightedCredentialTierStatus::Archived;
        assert!(
            item.move_to(WeightedCredentialTierStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
