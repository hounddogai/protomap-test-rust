use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary federation bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryFederationBundleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryFederationBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryFederationBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary federation bundle as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFederationBundle {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub attributes: BTreeMap<String, String>,
    pub title: String,
    pub cluster: String,
    pub active: bool,
    pub source: String,
    pub scope: String,
    pub port: i64,
    pub version: i64,
    pub status: PrimaryFederationBundleStatus,
}

impl PrimaryFederationBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryFederationBundleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryFederationBundleStatus) -> bool {
        match self.status {
            PrimaryFederationBundleStatus::Proposed => {
                return next == PrimaryFederationBundleStatus::Approved;
            }
            PrimaryFederationBundleStatus::Approved => {
                match next {
                    PrimaryFederationBundleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFederationBundleStatus::Rejected => {
                return next == PrimaryFederationBundleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryFederationBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary federation bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary federation bundle", self.code)
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

    pub fn net_value(&self) -> f64 {
        ((self.threshold + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryFederationBundle {
        PrimaryFederationBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            attributes: self.attributes.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            active: self.active,
            source: self.source.clone(),
            scope: self.scope.clone(),
            port: self.port,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary federation bundle in status proposed.
pub fn new_primary_federation_bundle(code: &str, name: &str) -> PrimaryFederationBundle {
    PrimaryFederationBundle {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        attributes: BTreeMap::new(),
        title: "".to_string(),
        cluster: "".to_string(),
        active: false,
        source: "".to_string(),
        scope: "".to_string(),
        port: 0,
        version: 0,
        status: PrimaryFederationBundleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_federation_bundle("code-413", "name-413");
        assert_eq!(item.status, PrimaryFederationBundleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_primary_federation_bundle("code-413", "name-413");
        if item.move_to(PrimaryFederationBundleStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryFederationBundleStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_federation_bundle("code-413", "name-413");
        item.status = PrimaryFederationBundleStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryFederationBundleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
