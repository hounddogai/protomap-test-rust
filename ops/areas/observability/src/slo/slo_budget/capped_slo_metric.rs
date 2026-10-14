use super::*;

use acme_kit::text;

/// Where a capped slo metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSloMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedSloMetricStatus {
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

impl std::fmt::Display for CappedSloMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped slo metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloMetric {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub limit: i64,
    pub grace_days: i64,
    pub rotation_days: i64,
    pub timezone: String,
    pub pinned: bool,
    pub margin: f64,
    pub active: bool,
    pub status: CappedSloMetricStatus,
}

impl CappedSloMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSloMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSloMetricStatus) -> bool {
        match self.status {
            CappedSloMetricStatus::Draft => {
                match next {
                    CappedSloMetricStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSloMetricStatus::Pending => {
                match next {
                    CappedSloMetricStatus::Active | CappedSloMetricStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSloMetricStatus::Active => {
                return next == CappedSloMetricStatus::Suspended || next == CappedSloMetricStatus::Closed;
            }
            CappedSloMetricStatus::Suspended => {
                return next == CappedSloMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedSloMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped slo metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.grace_days as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSloMetric {
        CappedSloMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            limit: self.limit,
            grace_days: self.grace_days,
            rotation_days: self.rotation_days,
            timezone: self.timezone.clone(),
            pinned: self.pinned,
            margin: self.margin,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped slo metric in status draft.
pub fn new_capped_slo_metric(code: &str, title: &str) -> CappedSloMetric {
    CappedSloMetric {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        limit: 0,
        grace_days: 0,
        rotation_days: 0,
        timezone: "".to_string(),
        pinned: false,
        margin: 0.0,
        active: false,
        status: CappedSloMetricStatus::Draft,
    }
}
