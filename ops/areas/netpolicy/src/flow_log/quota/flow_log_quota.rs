use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a flow log quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogQuotaStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FlowLogQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FlowLogQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log quota as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogQuota {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub attributes: BTreeMap<String, String>,
    pub updated_at: i64,
    pub attempts: i64,
    pub burst: i64,
    pub region: String,
    pub starts_at: i64,
    pub status: FlowLogQuotaStatus,
}

impl FlowLogQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogQuotaStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogQuotaStatus) -> bool {
        match self.status {
            FlowLogQuotaStatus::Proposed => {
                return next == FlowLogQuotaStatus::Rejected;
            }
            FlowLogQuotaStatus::Approved => {
                return next == FlowLogQuotaStatus::Withdrawn;
            }
            FlowLogQuotaStatus::Rejected => {
                return next == FlowLogQuotaStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FlowLogQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "flow log quota", self.code)
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

    pub fn weighted_exposure(&self) -> f64 {
        (self.score * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FlowLogQuota {
        FlowLogQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            attributes: self.attributes.clone(),
            updated_at: self.updated_at,
            attempts: self.attempts,
            burst: self.burst,
            region: self.region.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a flow log quota in status proposed.
pub fn new_flow_log_quota(code: &str, label: &str) -> FlowLogQuota {
    FlowLogQuota {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        attributes: BTreeMap::new(),
        updated_at: 0,
        attempts: 0,
        burst: 0,
        region: "".to_string(),
        starts_at: 0,
        status: FlowLogQuotaStatus::Proposed,
    }
}
