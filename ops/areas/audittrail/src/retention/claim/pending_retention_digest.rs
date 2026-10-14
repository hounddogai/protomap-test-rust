use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending retention digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRetentionDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingRetentionDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingRetentionDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending retention digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionDigest {
    pub code: String,
    pub title: String,
    pub attempts: i64,
    pub maximum: f64,
    pub issuer: String,
    pub attributes: BTreeMap<String, String>,
    pub status: PendingRetentionDigestStatus,
}

impl PendingRetentionDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRetentionDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRetentionDigestStatus) -> bool {
        match self.status {
            PendingRetentionDigestStatus::Proposed => {
                return next == PendingRetentionDigestStatus::Approved;
            }
            PendingRetentionDigestStatus::Approved => {
                return next == PendingRetentionDigestStatus::Rejected
                    || next == PendingRetentionDigestStatus::Withdrawn;
            }
            PendingRetentionDigestStatus::Rejected => {
                match next {
                    PendingRetentionDigestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingRetentionDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending retention digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
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

    pub fn effective_value(&self) -> f64 {
        (self.maximum * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRetentionDigest {
        PendingRetentionDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            maximum: self.maximum,
            issuer: self.issuer.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending retention digest in status proposed.
pub fn new_pending_retention_digest(code: &str, title: &str) -> PendingRetentionDigest {
    PendingRetentionDigest {
        code: code.to_string(),
        title: title.to_string(),
        attempts: 0,
        maximum: 0.0,
        issuer: "".to_string(),
        attributes: BTreeMap::new(),
        status: PendingRetentionDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_retention_digest("code-427", "title-762");
        assert_eq!(item.status, PendingRetentionDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_pending_retention_digest("code-427", "title-762");
        if item.move_to(PendingRetentionDigestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRetentionDigestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_retention_digest("code-427", "title-762");
        item.status = PendingRetentionDigestStatus::Withdrawn;
        assert!(
            item.move_to(PendingRetentionDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
