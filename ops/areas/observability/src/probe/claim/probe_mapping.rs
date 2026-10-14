use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a probe mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ProbeMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ProbeMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMapping {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub retries: i64,
    pub namespace: String,
    pub country: String,
    pub attributes: BTreeMap<String, String>,
    pub quota_units: i64,
    pub tags: Vec<String>,
    pub amount: f64,
    pub window_days: i64,
    pub status: ProbeMappingStatus,
}

impl ProbeMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeMappingStatus) -> bool {
        match self.status {
            ProbeMappingStatus::Requested => {
                return next == ProbeMappingStatus::Granted || next == ProbeMappingStatus::Expired;
            }
            ProbeMappingStatus::Granted => {
                match next {
                    ProbeMappingStatus::Revoked | ProbeMappingStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeMappingStatus::Revoked => {
                match next {
                    ProbeMappingStatus::Expired => {
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

    pub fn move_to(&mut self, next: ProbeMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.window_days as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeMapping {
        ProbeMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            retries: self.retries,
            namespace: self.namespace.clone(),
            country: self.country.clone(),
            attributes: self.attributes.clone(),
            quota_units: self.quota_units,
            tags: self.tags.clone(),
            amount: self.amount,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a probe mapping in status requested.
pub fn new_probe_mapping(code: &str, label: &str) -> ProbeMapping {
    ProbeMapping {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        retries: 0,
        namespace: "".to_string(),
        country: "".to_string(),
        attributes: BTreeMap::new(),
        quota_units: 0,
        tags: Vec::new(),
        amount: 0.0,
        window_days: 0,
        status: ProbeMappingStatus::Requested,
    }
}
