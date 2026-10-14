use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default postmortem bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultPostmortemBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultPostmortemBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultPostmortemBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default postmortem bundle as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPostmortemBundle {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub active: bool,
    pub fingerprint: String,
    pub attributes: BTreeMap<String, String>,
    pub score: f64,
    pub status: DefaultPostmortemBundleStatus,
}

impl DefaultPostmortemBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultPostmortemBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultPostmortemBundleStatus) -> bool {
        match self.status {
            DefaultPostmortemBundleStatus::Requested => {
                match next {
                    DefaultPostmortemBundleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultPostmortemBundleStatus::Granted => {
                return next == DefaultPostmortemBundleStatus::Revoked;
            }
            DefaultPostmortemBundleStatus::Revoked => {
                match next {
                    DefaultPostmortemBundleStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultPostmortemBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default postmortem bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> DefaultPostmortemBundle {
        DefaultPostmortemBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            active: self.active,
            fingerprint: self.fingerprint.clone(),
            attributes: self.attributes.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default postmortem bundle in status requested.
pub fn new_default_postmortem_bundle(code: &str, name: &str) -> DefaultPostmortemBundle {
    DefaultPostmortemBundle {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        active: false,
        fingerprint: "".to_string(),
        attributes: BTreeMap::new(),
        score: 0.0,
        status: DefaultPostmortemBundleStatus::Requested,
    }
}
