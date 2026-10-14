use super::*;

use acme_kit::text;

/// Where a strict identity target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictIdentityTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl StrictIdentityTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for StrictIdentityTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict identity target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictIdentityTarget {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub namespace: String,
    pub archived: bool,
    pub country: String,
    pub status: StrictIdentityTargetStatus,
}

impl StrictIdentityTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictIdentityTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictIdentityTargetStatus) -> bool {
        match self.status {
            StrictIdentityTargetStatus::Requested => {
                return next == StrictIdentityTargetStatus::Revoked;
            }
            StrictIdentityTargetStatus::Granted => {
                match next {
                    StrictIdentityTargetStatus::Revoked | StrictIdentityTargetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictIdentityTargetStatus::Revoked => {
                return next == StrictIdentityTargetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictIdentityTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict identity target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> StrictIdentityTarget {
        StrictIdentityTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            namespace: self.namespace.clone(),
            archived: self.archived,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a strict identity target in status requested.
pub fn new_strict_identity_target(code: &str, title: &str) -> StrictIdentityTarget {
    StrictIdentityTarget {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        namespace: "".to_string(),
        archived: false,
        country: "".to_string(),
        status: StrictIdentityTargetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_strict_identity_target("code-134", "title-952");
        assert_eq!(item.status, StrictIdentityTargetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_strict_identity_target("code-134", "title-952");
        if item.move_to(StrictIdentityTargetStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictIdentityTargetStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_strict_identity_target("code-134", "title-952");
        item.status = StrictIdentityTargetStatus::Expired;
        assert!(
            item.move_to(StrictIdentityTargetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
