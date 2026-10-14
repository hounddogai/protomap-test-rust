use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted query manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedQueryManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedQueryManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedQueryManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted query manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryManifest {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub channel: String,
    pub country: String,
    pub sequence: i64,
    pub created_at: i64,
    pub timezone: String,
    pub currency: String,
    pub description: String,
    pub attributes: BTreeMap<String, String>,
    pub status: WeightedQueryManifestStatus,
}

impl WeightedQueryManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedQueryManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedQueryManifestStatus) -> bool {
        match self.status {
            WeightedQueryManifestStatus::Proposed => {
                return next == WeightedQueryManifestStatus::Approved;
            }
            WeightedQueryManifestStatus::Approved => {
                match next {
                    WeightedQueryManifestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedQueryManifestStatus::Rejected => {
                return next == WeightedQueryManifestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedQueryManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted query manifest {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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

    pub fn projected_value(&self) -> f64 {
        ((self.sequence as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedQueryManifest {
        WeightedQueryManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            channel: self.channel.clone(),
            country: self.country.clone(),
            sequence: self.sequence,
            created_at: self.created_at,
            timezone: self.timezone.clone(),
            currency: self.currency.clone(),
            description: self.description.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a weighted query manifest in status proposed.
pub fn new_weighted_query_manifest(code: &str, label: &str) -> WeightedQueryManifest {
    WeightedQueryManifest {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        channel: "".to_string(),
        country: "".to_string(),
        sequence: 0,
        created_at: 0,
        timezone: "".to_string(),
        currency: "".to_string(),
        description: "".to_string(),
        attributes: BTreeMap::new(),
        status: WeightedQueryManifestStatus::Proposed,
    }
}
