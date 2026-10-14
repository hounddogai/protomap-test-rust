use super::*;

use acme_kit::text;

/// Where a weighted query profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedQueryProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedQueryProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedQueryProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted query profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryProfile {
    pub code: String,
    pub label: String,
    pub country: String,
    pub host: String,
    pub namespace: String,
    pub locale: String,
    pub ttl_seconds: i64,
    pub scope: String,
    pub attempts: i64,
    pub audience: String,
    pub status: WeightedQueryProfileStatus,
}

impl WeightedQueryProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedQueryProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedQueryProfileStatus) -> bool {
        match self.status {
            WeightedQueryProfileStatus::Proposed => {
                match next {
                    WeightedQueryProfileStatus::Approved | WeightedQueryProfileStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedQueryProfileStatus::Approved => {
                return next == WeightedQueryProfileStatus::Withdrawn;
            }
            WeightedQueryProfileStatus::Rejected => {
                match next {
                    WeightedQueryProfileStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: WeightedQueryProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted query profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.attempts as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedQueryProfile {
        WeightedQueryProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            host: self.host.clone(),
            namespace: self.namespace.clone(),
            locale: self.locale.clone(),
            ttl_seconds: self.ttl_seconds,
            scope: self.scope.clone(),
            attempts: self.attempts,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a weighted query profile in status proposed.
pub fn new_weighted_query_profile(code: &str, label: &str) -> WeightedQueryProfile {
    WeightedQueryProfile {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        host: "".to_string(),
        namespace: "".to_string(),
        locale: "".to_string(),
        ttl_seconds: 0,
        scope: "".to_string(),
        attempts: 0,
        audience: "".to_string(),
        status: WeightedQueryProfileStatus::Proposed,
    }
}
