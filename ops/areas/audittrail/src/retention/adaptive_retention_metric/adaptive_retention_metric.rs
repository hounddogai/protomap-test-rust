use super::*;

use acme_kit::text;

/// Where a adaptive retention metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRetentionMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveRetentionMetricStatus {
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

impl std::fmt::Display for AdaptiveRetentionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive retention metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRetentionMetric {
    pub code: String,
    pub title: String,
    pub label: String,
    pub segment: String,
    pub threshold: f64,
    pub score: f64,
    pub amount: f64,
    pub tags: Vec<String>,
    pub rotation_days: i64,
    pub status: AdaptiveRetentionMetricStatus,
}

impl AdaptiveRetentionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRetentionMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRetentionMetricStatus) -> bool {
        match self.status {
            AdaptiveRetentionMetricStatus::Planned => {
                match next {
                    AdaptiveRetentionMetricStatus::InProgress | AdaptiveRetentionMetricStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRetentionMetricStatus::Scheduled => {
                match next {
                    AdaptiveRetentionMetricStatus::InProgress | AdaptiveRetentionMetricStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRetentionMetricStatus::InProgress => {
                return next == AdaptiveRetentionMetricStatus::Cancelled;
            }
            AdaptiveRetentionMetricStatus::Completed => {
                match next {
                    AdaptiveRetentionMetricStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: AdaptiveRetentionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive retention metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive retention metric", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rotation_days as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRetentionMetric {
        AdaptiveRetentionMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            threshold: self.threshold,
            score: self.score,
            amount: self.amount,
            tags: self.tags.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a adaptive retention metric in status planned.
pub fn new_adaptive_retention_metric(code: &str, title: &str) -> AdaptiveRetentionMetric {
    AdaptiveRetentionMetric {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        segment: "".to_string(),
        threshold: 0.0,
        score: 0.0,
        amount: 0.0,
        tags: Vec::new(),
        rotation_days: 0,
        status: AdaptiveRetentionMetricStatus::Planned,
    }
}
