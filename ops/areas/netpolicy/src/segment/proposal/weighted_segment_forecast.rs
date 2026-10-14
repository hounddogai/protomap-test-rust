use super::*;

use acme_kit::text;

/// Where a weighted segment forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedSegmentForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedSegmentForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for WeightedSegmentForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted segment forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentForecast {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub tags: Vec<String>,
    pub timezone: String,
    pub maximum: f64,
    pub tenant_id: String,
    pub rotation_days: i64,
    pub priority: i64,
    pub status: WeightedSegmentForecastStatus,
}

impl WeightedSegmentForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedSegmentForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedSegmentForecastStatus) -> bool {
        match self.status {
            WeightedSegmentForecastStatus::New => {
                match next {
                    WeightedSegmentForecastStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSegmentForecastStatus::Queued => {
                return next == WeightedSegmentForecastStatus::Running || next == WeightedSegmentForecastStatus::Done;
            }
            WeightedSegmentForecastStatus::Running => {
                return next == WeightedSegmentForecastStatus::Failed;
            }
            WeightedSegmentForecastStatus::Done => {
                match next {
                    WeightedSegmentForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: WeightedSegmentForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted segment forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted segment forecast", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.priority as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedSegmentForecast {
        WeightedSegmentForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            tags: self.tags.clone(),
            timezone: self.timezone.clone(),
            maximum: self.maximum,
            tenant_id: self.tenant_id.clone(),
            rotation_days: self.rotation_days,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted segment forecast in status new.
pub fn new_weighted_segment_forecast(code: &str, title: &str) -> WeightedSegmentForecast {
    WeightedSegmentForecast {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        tags: Vec::new(),
        timezone: "".to_string(),
        maximum: 0.0,
        tenant_id: "".to_string(),
        rotation_days: 0,
        priority: 0,
        status: WeightedSegmentForecastStatus::New,
    }
}
