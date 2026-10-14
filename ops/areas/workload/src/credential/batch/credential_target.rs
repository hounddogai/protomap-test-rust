use super::*;

use acme_kit::text;

/// Where a credential target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CredentialTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CredentialTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialTarget {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub created_at: i64,
    pub cluster: String,
    pub attempts: i64,
    pub version: i64,
    pub scope: String,
    pub status: CredentialTargetStatus,
}

impl CredentialTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialTargetStatus) -> bool {
        match self.status {
            CredentialTargetStatus::Requested => {
                match next {
                    CredentialTargetStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialTargetStatus::Granted => {
                return next == CredentialTargetStatus::Expired;
            }
            CredentialTargetStatus::Revoked => {
                return next == CredentialTargetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialTarget {
        CredentialTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            created_at: self.created_at,
            cluster: self.cluster.clone(),
            attempts: self.attempts,
            version: self.version,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a credential target in status requested.
pub fn new_credential_target(code: &str, name: &str) -> CredentialTarget {
    CredentialTarget {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        created_at: 0,
        cluster: "".to_string(),
        attempts: 0,
        version: 0,
        scope: "".to_string(),
        status: CredentialTargetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_credential_target("code-664", "name-278");
        assert_eq!(item.status, CredentialTargetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_credential_target("code-664", "name-278");
        if item.move_to(CredentialTargetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialTargetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_credential_target("code-664", "name-278");
        item.status = CredentialTargetStatus::Expired;
        assert!(
            item.move_to(CredentialTargetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
