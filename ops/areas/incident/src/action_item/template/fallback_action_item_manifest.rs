use super::*;

use acme_kit::text;

/// Where a fallback action item manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackActionItemManifestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackActionItemManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackActionItemManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback action item manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemManifest {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub maximum: f64,
    pub approved: bool,
    pub budget: f64,
    pub status: FallbackActionItemManifestStatus,
}

impl FallbackActionItemManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackActionItemManifestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackActionItemManifestStatus) -> bool {
        match self.status {
            FallbackActionItemManifestStatus::Requested => {
                match next {
                    FallbackActionItemManifestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackActionItemManifestStatus::Granted => {
                return next == FallbackActionItemManifestStatus::Revoked;
            }
            FallbackActionItemManifestStatus::Revoked => {
                match next {
                    FallbackActionItemManifestStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackActionItemManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback action item manifest {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "fallback action item manifest", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.expires_at as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackActionItemManifest {
        FallbackActionItemManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            maximum: self.maximum,
            approved: self.approved,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback action item manifest in status requested.
pub fn new_fallback_action_item_manifest(code: &str, name: &str) -> FallbackActionItemManifest {
    FallbackActionItemManifest {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        maximum: 0.0,
        approved: false,
        budget: 0.0,
        status: FallbackActionItemManifestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_action_item_manifest("code-338", "name-24");
        assert_eq!(item.status, FallbackActionItemManifestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_fallback_action_item_manifest("code-338", "name-24");
        if item.move_to(FallbackActionItemManifestStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackActionItemManifestStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_action_item_manifest("code-338", "name-24");
        item.status = FallbackActionItemManifestStatus::Expired;
        assert!(
            item.move_to(FallbackActionItemManifestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
