use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending identity override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingIdentityOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingIdentityOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingIdentityOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending identity override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIdentityOverride {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub owner_id: String,
    pub scope: String,
    pub created_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub window_days: i64,
    pub status: PendingIdentityOverrideStatus,
}

impl PendingIdentityOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingIdentityOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingIdentityOverrideStatus) -> bool {
        match self.status {
            PendingIdentityOverrideStatus::Created => {
                match next {
                    PendingIdentityOverrideStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingIdentityOverrideStatus::Validated => {
                return next == PendingIdentityOverrideStatus::Published
                    || next == PendingIdentityOverrideStatus::Retired;
            }
            PendingIdentityOverrideStatus::Published => {
                return next == PendingIdentityOverrideStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingIdentityOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending identity override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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
        ((self.created_at as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingIdentityOverride {
        PendingIdentityOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            owner_id: self.owner_id.clone(),
            scope: self.scope.clone(),
            created_at: self.created_at,
            attributes: self.attributes.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a pending identity override in status created.
pub fn new_pending_identity_override(code: &str, name: &str) -> PendingIdentityOverride {
    PendingIdentityOverride {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        owner_id: "".to_string(),
        scope: "".to_string(),
        created_at: 0,
        attributes: BTreeMap::new(),
        window_days: 0,
        status: PendingIdentityOverrideStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_identity_override("code-141", "name-46");
        assert_eq!(item.status, PendingIdentityOverrideStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_pending_identity_override("code-141", "name-46");
        if item.move_to(PendingIdentityOverrideStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingIdentityOverrideStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_identity_override("code-141", "name-46");
        item.status = PendingIdentityOverrideStatus::Retired;
        assert!(
            item.move_to(PendingIdentityOverrideStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
