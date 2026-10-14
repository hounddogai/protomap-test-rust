use super::*;

use acme_kit::text;

/// Where a identity manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IdentityManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IdentityManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One identity manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityManifest {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub checksum: String,
    pub revision: i64,
    pub cluster: String,
    pub attempts: i64,
    pub host: String,
    pub status: IdentityManifestStatus,
}

impl IdentityManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityManifestStatus) -> bool {
        match self.status {
            IdentityManifestStatus::Requested => {
                match next {
                    IdentityManifestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityManifestStatus::Granted => {
                return next == IdentityManifestStatus::Revoked;
            }
            IdentityManifestStatus::Revoked => {
                return next == IdentityManifestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity manifest {} cannot move from {} to {}",
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.attempts as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityManifest {
        IdentityManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            checksum: self.checksum.clone(),
            revision: self.revision,
            cluster: self.cluster.clone(),
            attempts: self.attempts,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a identity manifest in status requested.
pub fn new_identity_manifest(code: &str, title: &str) -> IdentityManifest {
    IdentityManifest {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        checksum: "".to_string(),
        revision: 0,
        cluster: "".to_string(),
        attempts: 0,
        host: "".to_string(),
        status: IdentityManifestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_identity_manifest("code-863", "title-323");
        assert_eq!(item.status, IdentityManifestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_identity_manifest("code-863", "title-323");
        if item.move_to(IdentityManifestStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityManifestStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_identity_manifest("code-863", "title-323");
        item.status = IdentityManifestStatus::Expired;
        assert!(
            item.move_to(IdentityManifestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
