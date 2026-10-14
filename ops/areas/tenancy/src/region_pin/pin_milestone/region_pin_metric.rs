use super::*;

use acme_kit::text;

/// Where a region pin metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionPinMetricStatus {
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

impl std::fmt::Display for RegionPinMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMetric {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub fingerprint: String,
    pub score: f64,
    pub pinned: bool,
    pub unit: String,
    pub batch_size: i64,
    pub reviewer: String,
    pub status: RegionPinMetricStatus,
}

impl RegionPinMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinMetricStatus) -> bool {
        match self.status {
            RegionPinMetricStatus::Draft => {
                return next == RegionPinMetricStatus::Pending;
            }
            RegionPinMetricStatus::Pending => {
                match next {
                    RegionPinMetricStatus::Suspended | RegionPinMetricStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinMetricStatus::Active => {
                return next == RegionPinMetricStatus::Suspended || next == RegionPinMetricStatus::Closed;
            }
            RegionPinMetricStatus::Suspended => {
                return next == RegionPinMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "region pin metric", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.score + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinMetric {
        RegionPinMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            fingerprint: self.fingerprint.clone(),
            score: self.score,
            pinned: self.pinned,
            unit: self.unit.clone(),
            batch_size: self.batch_size,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a region pin metric in status draft.
pub fn new_region_pin_metric(code: &str, label: &str) -> RegionPinMetric {
    RegionPinMetric {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        fingerprint: "".to_string(),
        score: 0.0,
        pinned: false,
        unit: "".to_string(),
        batch_size: 0,
        reviewer: "".to_string(),
        status: RegionPinMetricStatus::Draft,
    }
}
