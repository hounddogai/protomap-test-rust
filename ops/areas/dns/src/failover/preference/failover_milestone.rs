use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a failover milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverMilestoneStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FailoverMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FailoverMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverMilestone {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub region: String,
    pub quota_units: i64,
    pub port: i64,
    pub ends_at: i64,
    pub key_id: String,
    pub ttl_seconds: i64,
    pub status: FailoverMilestoneStatus,
}

impl FailoverMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverMilestoneStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverMilestoneStatus) -> bool {
        match self.status {
            FailoverMilestoneStatus::Created => {
                return next == FailoverMilestoneStatus::Published;
            }
            FailoverMilestoneStatus::Validated => {
                match next {
                    FailoverMilestoneStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverMilestoneStatus::Published => {
                match next {
                    FailoverMilestoneStatus::Retired => {
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

    pub fn move_to(&mut self, next: FailoverMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover milestone {} cannot move from {} to {}",
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
        format!("{} {}", "failover milestone", self.code)
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
        ((self.ends_at as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverMilestone {
        FailoverMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            region: self.region.clone(),
            quota_units: self.quota_units,
            port: self.port,
            ends_at: self.ends_at,
            key_id: self.key_id.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a failover milestone in status created.
pub fn new_failover_milestone(code: &str, label: &str) -> FailoverMilestone {
    FailoverMilestone {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        region: "".to_string(),
        quota_units: 0,
        port: 0,
        ends_at: 0,
        key_id: "".to_string(),
        ttl_seconds: 0,
        status: FailoverMilestoneStatus::Created,
    }
}
