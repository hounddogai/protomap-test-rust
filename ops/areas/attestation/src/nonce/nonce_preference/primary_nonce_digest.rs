use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary nonce digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryNonceDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryNonceDigestStatus {
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

impl std::fmt::Display for PrimaryNonceDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary nonce digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceDigest {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub limit: i64,
    pub attributes: BTreeMap<String, String>,
    pub segment: String,
    pub external_ref: String,
    pub status: PrimaryNonceDigestStatus,
}

impl PrimaryNonceDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryNonceDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryNonceDigestStatus) -> bool {
        match self.status {
            PrimaryNonceDigestStatus::Open => {
                match next {
                    PrimaryNonceDigestStatus::InReview | PrimaryNonceDigestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryNonceDigestStatus::InReview => {
                return next == PrimaryNonceDigestStatus::Resolved || next == PrimaryNonceDigestStatus::Archived;
            }
            PrimaryNonceDigestStatus::Resolved => {
                match next {
                    PrimaryNonceDigestStatus::Reopened | PrimaryNonceDigestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryNonceDigestStatus::Reopened => {
                match next {
                    PrimaryNonceDigestStatus::Archived => {
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

    pub fn move_to(&mut self, next: PrimaryNonceDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary nonce digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary nonce digest", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> PrimaryNonceDigest {
        PrimaryNonceDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            limit: self.limit,
            attributes: self.attributes.clone(),
            segment: self.segment.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a primary nonce digest in status open.
pub fn new_primary_nonce_digest(code: &str, name: &str) -> PrimaryNonceDigest {
    PrimaryNonceDigest {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        limit: 0,
        attributes: BTreeMap::new(),
        segment: "".to_string(),
        external_ref: "".to_string(),
        status: PrimaryNonceDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_primary_nonce_digest("code-742", "name-251");
        assert_eq!(item.status, PrimaryNonceDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_primary_nonce_digest("code-742", "name-251");
        if item.move_to(PrimaryNonceDigestStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryNonceDigestStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_primary_nonce_digest("code-742", "name-251");
        item.status = PrimaryNonceDigestStatus::Archived;
        assert!(
            item.move_to(PrimaryNonceDigestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
