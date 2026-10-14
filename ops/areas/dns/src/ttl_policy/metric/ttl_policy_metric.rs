use super::*;

use acme_kit::text;

/// Where a ttl policy metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TtlPolicyMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TtlPolicyMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy metric as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMetric {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub contact_email: String,
    pub floor_value: f64,
    pub issuer: String,
    pub principal: String,
    pub pinned: bool,
    pub rotation_days: i64,
    pub source: String,
    pub status: TtlPolicyMetricStatus,
}

impl TtlPolicyMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyMetricStatus) -> bool {
        match self.status {
            TtlPolicyMetricStatus::Proposed => {
                match next {
                    TtlPolicyMetricStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyMetricStatus::Approved => {
                match next {
                    TtlPolicyMetricStatus::Rejected | TtlPolicyMetricStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyMetricStatus::Rejected => {
                return next == TtlPolicyMetricStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "ttl policy metric", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.rotation_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyMetric {
        TtlPolicyMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            contact_email: self.contact_email.clone(),
            floor_value: self.floor_value,
            issuer: self.issuer.clone(),
            principal: self.principal.clone(),
            pinned: self.pinned,
            rotation_days: self.rotation_days,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a ttl policy metric in status proposed.
pub fn new_ttl_policy_metric(code: &str, name: &str) -> TtlPolicyMetric {
    TtlPolicyMetric {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        contact_email: "".to_string(),
        floor_value: 0.0,
        issuer: "".to_string(),
        principal: "".to_string(),
        pinned: false,
        rotation_days: 0,
        source: "".to_string(),
        status: TtlPolicyMetricStatus::Proposed,
    }
}
