use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a tiered flow log preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredFlowLogPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredFlowLogPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for TieredFlowLogPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered flow log preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogPreference {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub notes: String,
    pub namespace: String,
    pub segment: String,
    pub visible: bool,
    pub minimum: f64,
    pub attributes: BTreeMap<String, String>,
    pub cluster: String,
    pub active: bool,
    pub status: TieredFlowLogPreferenceStatus,
}

impl TieredFlowLogPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredFlowLogPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredFlowLogPreferenceStatus) -> bool {
        match self.status {
            TieredFlowLogPreferenceStatus::Draft => {
                return next == TieredFlowLogPreferenceStatus::Active;
            }
            TieredFlowLogPreferenceStatus::Pending => {
                match next {
                    TieredFlowLogPreferenceStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFlowLogPreferenceStatus::Active => {
                match next {
                    TieredFlowLogPreferenceStatus::Suspended | TieredFlowLogPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFlowLogPreferenceStatus::Suspended => {
                return next == TieredFlowLogPreferenceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredFlowLogPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered flow log preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn projected_load(&self) -> f64 {
        ((self.retries as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredFlowLogPreference {
        TieredFlowLogPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            notes: self.notes.clone(),
            namespace: self.namespace.clone(),
            segment: self.segment.clone(),
            visible: self.visible,
            minimum: self.minimum,
            attributes: self.attributes.clone(),
            cluster: self.cluster.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a tiered flow log preference in status draft.
pub fn new_tiered_flow_log_preference(code: &str, label: &str) -> TieredFlowLogPreference {
    TieredFlowLogPreference {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        notes: "".to_string(),
        namespace: "".to_string(),
        segment: "".to_string(),
        visible: false,
        minimum: 0.0,
        attributes: BTreeMap::new(),
        cluster: "".to_string(),
        active: false,
        status: TieredFlowLogPreferenceStatus::Draft,
    }
}
