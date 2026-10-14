use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a silence bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceBundleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SilenceBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SilenceBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One silence bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceBundle {
    pub code: String,
    pub title: String,
    pub name: String,
    pub country: String,
    pub attributes: BTreeMap<String, String>,
    pub expires_at: i64,
    pub burst: i64,
    pub capacity: i64,
    pub created_at: i64,
    pub status: SilenceBundleStatus,
}

impl SilenceBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceBundleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceBundleStatus) -> bool {
        match self.status {
            SilenceBundleStatus::Requested => {
                return next == SilenceBundleStatus::Expired;
            }
            SilenceBundleStatus::Granted => {
                match next {
                    SilenceBundleStatus::Revoked | SilenceBundleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceBundleStatus::Revoked => {
                return next == SilenceBundleStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence bundle {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
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

    pub fn net_volume(&self) -> f64 {
        ((self.expires_at as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceBundle {
        SilenceBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            attributes: self.attributes.clone(),
            expires_at: self.expires_at,
            burst: self.burst,
            capacity: self.capacity,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a silence bundle in status requested.
pub fn new_silence_bundle(code: &str, title: &str) -> SilenceBundle {
    SilenceBundle {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        country: "".to_string(),
        attributes: BTreeMap::new(),
        expires_at: 0,
        burst: 0,
        capacity: 0,
        created_at: 0,
        status: SilenceBundleStatus::Requested,
    }
}
