use super::*;

use acme_kit::text;

/// Where a segment metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SegmentMetricStatus {
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

impl std::fmt::Display for SegmentMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment metric as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMetric {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub phone: String,
    pub unit: String,
    pub threshold: f64,
    pub currency: String,
    pub revision: i64,
    pub attempts: i64,
    pub scope: String,
    pub region: String,
    pub status: SegmentMetricStatus,
}

impl SegmentMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentMetricStatus) -> bool {
        match self.status {
            SegmentMetricStatus::New => {
                match next {
                    SegmentMetricStatus::Queued | SegmentMetricStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentMetricStatus::Queued => {
                match next {
                    SegmentMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentMetricStatus::Running => {
                return next == SegmentMetricStatus::Done;
            }
            SegmentMetricStatus::Done => {
                return next == SegmentMetricStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.attempts as f64 + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentMetric {
        SegmentMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            phone: self.phone.clone(),
            unit: self.unit.clone(),
            threshold: self.threshold,
            currency: self.currency.clone(),
            revision: self.revision,
            attempts: self.attempts,
            scope: self.scope.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a segment metric in status new.
pub fn new_segment_metric(code: &str, title: &str) -> SegmentMetric {
    SegmentMetric {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        phone: "".to_string(),
        unit: "".to_string(),
        threshold: 0.0,
        currency: "".to_string(),
        revision: 0,
        attempts: 0,
        scope: "".to_string(),
        region: "".to_string(),
        status: SegmentMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_segment_metric("code-821", "title-624");
        assert_eq!(item.status, SegmentMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_segment_metric("code-821", "title-624");
        if item.move_to(SegmentMetricStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentMetricStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_segment_metric("code-821", "title-624");
        item.status = SegmentMetricStatus::Failed;
        assert!(item.move_to(SegmentMetricStatus::New).is_err(), "expected move to fail");
    }
}
