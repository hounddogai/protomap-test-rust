use super::*;

use acme_kit::text;

/// Where a policy manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyManifestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PolicyManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PolicyManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyManifest {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub expires_at: i64,
    pub external_ref: String,
    pub ttl_seconds: i64,
    pub status: PolicyManifestStatus,
}

impl PolicyManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyManifestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyManifestStatus) -> bool {
        match self.status {
            PolicyManifestStatus::Created => {
                match next {
                    PolicyManifestStatus::Published | PolicyManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyManifestStatus::Validated => {
                match next {
                    PolicyManifestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyManifestStatus::Published => {
                return next == PolicyManifestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "policy manifest", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyManifest {
        PolicyManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a policy manifest in status created.
pub fn new_policy_manifest(code: &str, label: &str) -> PolicyManifest {
    PolicyManifest {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        expires_at: 0,
        external_ref: "".to_string(),
        ttl_seconds: 0,
        status: PolicyManifestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_policy_manifest("code-900", "label-528");
        assert_eq!(item.status, PolicyManifestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_policy_manifest("code-900", "label-528");
        if item.move_to(PolicyManifestStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyManifestStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_policy_manifest("code-900", "label-528");
        item.status = PolicyManifestStatus::Retired;
        assert!(
            item.move_to(PolicyManifestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
