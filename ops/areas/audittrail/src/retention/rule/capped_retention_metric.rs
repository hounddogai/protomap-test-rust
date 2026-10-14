use super::*;

use acme_kit::text;

/// Where a capped retention metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRetentionMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedRetentionMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedRetentionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped retention metric as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionMetric {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub region: String,
    pub limit: i64,
    pub status: CappedRetentionMetricStatus,
}

impl CappedRetentionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRetentionMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRetentionMetricStatus) -> bool {
        match self.status {
            CappedRetentionMetricStatus::Proposed => {
                return next == CappedRetentionMetricStatus::Approved;
            }
            CappedRetentionMetricStatus::Approved => {
                return next == CappedRetentionMetricStatus::Rejected || next == CappedRetentionMetricStatus::Withdrawn;
            }
            CappedRetentionMetricStatus::Rejected => {
                return next == CappedRetentionMetricStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRetentionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped retention metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> CappedRetentionMetric {
        CappedRetentionMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            region: self.region.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a capped retention metric in status proposed.
pub fn new_capped_retention_metric(code: &str, label: &str) -> CappedRetentionMetric {
    CappedRetentionMetric {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        region: "".to_string(),
        limit: 0,
        status: CappedRetentionMetricStatus::Proposed,
    }
}
