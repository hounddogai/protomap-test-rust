use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback identity override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackIdentityOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackIdentityOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for FallbackIdentityOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback identity override as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackIdentityOverride {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub capacity: i64,
    pub attributes: BTreeMap<String, String>,
    pub rotation_days: i64,
    pub status: FallbackIdentityOverrideStatus,
}

impl FallbackIdentityOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackIdentityOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackIdentityOverrideStatus) -> bool {
        match self.status {
            FallbackIdentityOverrideStatus::Open => {
                return next == FallbackIdentityOverrideStatus::InReview
                    || next == FallbackIdentityOverrideStatus::Archived;
            }
            FallbackIdentityOverrideStatus::InReview => {
                match next {
                    FallbackIdentityOverrideStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackIdentityOverrideStatus::Resolved => {
                return next == FallbackIdentityOverrideStatus::Reopened
                    || next == FallbackIdentityOverrideStatus::Archived;
            }
            FallbackIdentityOverrideStatus::Reopened => {
                match next {
                    FallbackIdentityOverrideStatus::Archived => {
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

    pub fn move_to(&mut self, next: FallbackIdentityOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback identity override {} cannot move from {} to {}",
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
        format!("{} {}", "fallback identity override", self.code)
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

    pub fn weighted_total(&self) -> f64 {
        ((self.capacity as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackIdentityOverride {
        FallbackIdentityOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            capacity: self.capacity,
            attributes: self.attributes.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback identity override in status open.
pub fn new_fallback_identity_override(code: &str, name: &str) -> FallbackIdentityOverride {
    FallbackIdentityOverride {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        capacity: 0,
        attributes: BTreeMap::new(),
        rotation_days: 0,
        status: FallbackIdentityOverrideStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_identity_override("code-35", "name-987");
        assert_eq!(item.status, FallbackIdentityOverrideStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_fallback_identity_override("code-35", "name-987");
        if item.move_to(FallbackIdentityOverrideStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackIdentityOverrideStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_identity_override("code-35", "name-987");
        item.status = FallbackIdentityOverrideStatus::Archived;
        assert!(
            item.move_to(FallbackIdentityOverrideStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
