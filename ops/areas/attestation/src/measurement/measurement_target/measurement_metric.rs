use super::*;

use acme_kit::text;

/// Where a measurement metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl MeasurementMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for MeasurementMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One measurement metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMetric {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub source: String,
    pub port: i64,
    pub summary: String,
    pub scope: String,
    pub status: MeasurementMetricStatus,
}

impl MeasurementMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementMetricStatus) -> bool {
        match self.status {
            MeasurementMetricStatus::Planned => {
                match next {
                    MeasurementMetricStatus::Scheduled | MeasurementMetricStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementMetricStatus::Scheduled => {
                return next == MeasurementMetricStatus::InProgress || next == MeasurementMetricStatus::Cancelled;
            }
            MeasurementMetricStatus::InProgress => {
                match next {
                    MeasurementMetricStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementMetricStatus::Completed => {
                match next {
                    MeasurementMetricStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: MeasurementMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement metric", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.port as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementMetric {
        MeasurementMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            source: self.source.clone(),
            port: self.port,
            summary: self.summary.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a measurement metric in status planned.
pub fn new_measurement_metric(code: &str, title: &str) -> MeasurementMetric {
    MeasurementMetric {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        source: "".to_string(),
        port: 0,
        summary: "".to_string(),
        scope: "".to_string(),
        status: MeasurementMetricStatus::Planned,
    }
}
