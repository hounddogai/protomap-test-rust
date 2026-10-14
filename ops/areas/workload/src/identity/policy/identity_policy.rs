use super::*;

use acme_kit::text;

/// Where a identity policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IdentityPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IdentityPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityPolicy {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub rotation_days: i64,
    pub host: String,
    pub status: IdentityPolicyStatus,
}

impl IdentityPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityPolicyStatus) -> bool {
        match self.status {
            IdentityPolicyStatus::Requested => {
                match next {
                    IdentityPolicyStatus::Granted | IdentityPolicyStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityPolicyStatus::Granted => {
                match next {
                    IdentityPolicyStatus::Revoked | IdentityPolicyStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityPolicyStatus::Revoked => {
                return next == IdentityPolicyStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.floor_value - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityPolicy {
        IdentityPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            rotation_days: self.rotation_days,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a identity policy in status requested.
pub fn new_identity_policy(code: &str, name: &str) -> IdentityPolicy {
    IdentityPolicy {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        rotation_days: 0,
        host: "".to_string(),
        status: IdentityPolicyStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_identity_policy("code-700", "name-425");
        assert_eq!(item.status, IdentityPolicyStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_identity_policy("code-700", "name-425");
        if item.move_to(IdentityPolicyStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityPolicyStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_identity_policy("code-700", "name-425");
        item.status = IdentityPolicyStatus::Expired;
        assert!(
            item.move_to(IdentityPolicyStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
