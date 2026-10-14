use super::*;

use acme_kit::text;

/// Where a rollback digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollbackDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollbackDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollback digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDigest {
    pub code: String,
    pub label: String,
    pub owner_id: String,
    pub tags: Vec<String>,
    pub updated_at: i64,
    pub external_ref: String,
    pub status: RollbackDigestStatus,
}

impl RollbackDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackDigestStatus) -> bool {
        match self.status {
            RollbackDigestStatus::Requested => {
                match next {
                    RollbackDigestStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackDigestStatus::Granted => {
                return next == RollbackDigestStatus::Revoked;
            }
            RollbackDigestStatus::Revoked => {
                match next {
                    RollbackDigestStatus::Expired => {
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

    pub fn move_to(&mut self, next: RollbackDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback digest", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> RollbackDigest {
        RollbackDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            owner_id: self.owner_id.clone(),
            tags: self.tags.clone(),
            updated_at: self.updated_at,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollback digest in status requested.
pub fn new_rollback_digest(code: &str, label: &str) -> RollbackDigest {
    RollbackDigest {
        code: code.to_string(),
        label: label.to_string(),
        owner_id: "".to_string(),
        tags: Vec::new(),
        updated_at: 0,
        external_ref: "".to_string(),
        status: RollbackDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollback_digest("code-898", "label-411");
        assert_eq!(item.status, RollbackDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rollback_digest("code-898", "label-411");
        if item.move_to(RollbackDigestStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackDigestStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollback_digest("code-898", "label-411");
        item.status = RollbackDigestStatus::Expired;
        assert!(
            item.move_to(RollbackDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
