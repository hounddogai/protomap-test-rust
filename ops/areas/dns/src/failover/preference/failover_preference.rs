use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a failover preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FailoverPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FailoverPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverPreference {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub contact_email: String,
    pub timezone: String,
    pub grace_days: i64,
    pub cluster: String,
    pub region: String,
    pub attributes: BTreeMap<String, String>,
    pub status: FailoverPreferenceStatus,
}

impl FailoverPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverPreferenceStatus) -> bool {
        match self.status {
            FailoverPreferenceStatus::Proposed => {
                match next {
                    FailoverPreferenceStatus::Approved | FailoverPreferenceStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverPreferenceStatus::Approved => {
                match next {
                    FailoverPreferenceStatus::Rejected | FailoverPreferenceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverPreferenceStatus::Rejected => {
                return next == FailoverPreferenceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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
        ((self.grace_days as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverPreference {
        FailoverPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            contact_email: self.contact_email.clone(),
            timezone: self.timezone.clone(),
            grace_days: self.grace_days,
            cluster: self.cluster.clone(),
            region: self.region.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a failover preference in status proposed.
pub fn new_failover_preference(code: &str, title: &str) -> FailoverPreference {
    FailoverPreference {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        contact_email: "".to_string(),
        timezone: "".to_string(),
        grace_days: 0,
        cluster: "".to_string(),
        region: "".to_string(),
        attributes: BTreeMap::new(),
        status: FailoverPreferenceStatus::Proposed,
    }
}
