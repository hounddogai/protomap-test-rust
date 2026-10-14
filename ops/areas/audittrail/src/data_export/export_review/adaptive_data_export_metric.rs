use super::*;

use acme_kit::text;

/// Where a adaptive data export metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveDataExportMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveDataExportMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for AdaptiveDataExportMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive data export metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDataExportMetric {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub summary: String,
    pub priority: i64,
    pub status: AdaptiveDataExportMetricStatus,
}

impl AdaptiveDataExportMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveDataExportMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveDataExportMetricStatus) -> bool {
        match self.status {
            AdaptiveDataExportMetricStatus::Open => {
                match next {
                    AdaptiveDataExportMetricStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDataExportMetricStatus::InReview => {
                match next {
                    AdaptiveDataExportMetricStatus::Reopened | AdaptiveDataExportMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDataExportMetricStatus::Resolved => {
                match next {
                    AdaptiveDataExportMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDataExportMetricStatus::Reopened => {
                match next {
                    AdaptiveDataExportMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveDataExportMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive data export metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.priority as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveDataExportMetric {
        AdaptiveDataExportMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            summary: self.summary.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive data export metric in status open.
pub fn new_adaptive_data_export_metric(code: &str, label: &str) -> AdaptiveDataExportMetric {
    AdaptiveDataExportMetric {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        summary: "".to_string(),
        priority: 0,
        status: AdaptiveDataExportMetricStatus::Open,
    }
}
