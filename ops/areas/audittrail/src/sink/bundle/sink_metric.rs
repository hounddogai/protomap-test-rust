use super::*;

use acme_kit::text;

/// Where a sink metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkMetricStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SinkMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SinkMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink metric as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkMetric {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub duration_minutes: i64,
    pub quantity: i64,
    pub cost: f64,
    pub cluster: String,
    pub budget: f64,
    pub status: SinkMetricStatus,
}

impl SinkMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkMetricStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkMetricStatus) -> bool {
        match self.status {
            SinkMetricStatus::Requested => {
                return next == SinkMetricStatus::Granted || next == SinkMetricStatus::Revoked;
            }
            SinkMetricStatus::Granted => {
                match next {
                    SinkMetricStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkMetricStatus::Revoked => {
                return next == SinkMetricStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        (self.budget * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkMetric {
        SinkMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            duration_minutes: self.duration_minutes,
            quantity: self.quantity,
            cost: self.cost,
            cluster: self.cluster.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a sink metric in status requested.
pub fn new_sink_metric(code: &str, label: &str) -> SinkMetric {
    SinkMetric {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        duration_minutes: 0,
        quantity: 0,
        cost: 0.0,
        cluster: "".to_string(),
        budget: 0.0,
        status: SinkMetricStatus::Requested,
    }
}
