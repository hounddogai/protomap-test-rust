use super::*;

use acme_kit::text;

/// Where a manual nonce manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualNonceManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualNonceManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualNonceManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual nonce manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceManifest {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub tenant_id: String,
    pub cost: f64,
    pub notes: String,
    pub segment: String,
    pub phone: String,
    pub starts_at: i64,
    pub status: ManualNonceManifestStatus,
}

impl ManualNonceManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualNonceManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualNonceManifestStatus) -> bool {
        match self.status {
            ManualNonceManifestStatus::Requested => {
                return next == ManualNonceManifestStatus::Revoked;
            }
            ManualNonceManifestStatus::Granted => {
                return next == ManualNonceManifestStatus::Revoked || next == ManualNonceManifestStatus::Expired;
            }
            ManualNonceManifestStatus::Revoked => {
                return next == ManualNonceManifestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualNonceManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual nonce manifest {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.starts_at as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualNonceManifest {
        ManualNonceManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            tenant_id: self.tenant_id.clone(),
            cost: self.cost,
            notes: self.notes.clone(),
            segment: self.segment.clone(),
            phone: self.phone.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a manual nonce manifest in status requested.
pub fn new_manual_nonce_manifest(code: &str, title: &str) -> ManualNonceManifest {
    ManualNonceManifest {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        tenant_id: "".to_string(),
        cost: 0.0,
        notes: "".to_string(),
        segment: "".to_string(),
        phone: "".to_string(),
        starts_at: 0,
        status: ManualNonceManifestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_nonce_manifest("code-9", "title-43");
        assert_eq!(item.status, ManualNonceManifestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_manual_nonce_manifest("code-9", "title-43");
        if item.move_to(ManualNonceManifestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualNonceManifestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_nonce_manifest("code-9", "title-43");
        item.status = ManualNonceManifestStatus::Expired;
        assert!(
            item.move_to(ManualNonceManifestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
