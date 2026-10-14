use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback egress threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackEgressThresholdStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackEgressThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackEgressThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback egress threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressThreshold {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub archived: bool,
    pub ends_at: i64,
    pub status: FallbackEgressThresholdStatus,
}

impl FallbackEgressThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackEgressThresholdStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackEgressThresholdStatus) -> bool {
        match self.status {
            FallbackEgressThresholdStatus::Requested => {
                match next {
                    FallbackEgressThresholdStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEgressThresholdStatus::Granted => {
                match next {
                    FallbackEgressThresholdStatus::Revoked | FallbackEgressThresholdStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEgressThresholdStatus::Revoked => {
                match next {
                    FallbackEgressThresholdStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackEgressThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback egress threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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
        ((self.margin - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackEgressThreshold {
        FallbackEgressThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            archived: self.archived,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback egress threshold in status requested.
pub fn new_fallback_egress_threshold(code: &str, name: &str) -> FallbackEgressThreshold {
    FallbackEgressThreshold {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        label: "".to_string(),
        attributes: BTreeMap::new(),
        archived: false,
        ends_at: 0,
        status: FallbackEgressThresholdStatus::Requested,
    }
}
