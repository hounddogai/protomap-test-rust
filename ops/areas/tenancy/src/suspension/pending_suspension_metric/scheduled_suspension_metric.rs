use super::*;

use acme_kit::text;

/// Where a scheduled suspension metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSuspensionMetricStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledSuspensionMetricStatus {
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

impl std::fmt::Display for ScheduledSuspensionMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled suspension metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionMetric {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub version: i64,
    pub approved: bool,
    pub retries: i64,
    pub status: ScheduledSuspensionMetricStatus,
}

impl ScheduledSuspensionMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSuspensionMetricStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSuspensionMetricStatus) -> bool {
        match self.status {
            ScheduledSuspensionMetricStatus::New => {
                return next == ScheduledSuspensionMetricStatus::Queued;
            }
            ScheduledSuspensionMetricStatus::Queued => {
                match next {
                    ScheduledSuspensionMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSuspensionMetricStatus::Running => {
                match next {
                    ScheduledSuspensionMetricStatus::Done | ScheduledSuspensionMetricStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSuspensionMetricStatus::Done => {
                match next {
                    ScheduledSuspensionMetricStatus::Failed => {
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

    pub fn move_to(&mut self, next: ScheduledSuspensionMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled suspension metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled suspension metric", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.version as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSuspensionMetric {
        ScheduledSuspensionMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            version: self.version,
            approved: self.approved,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a scheduled suspension metric in status new.
pub fn new_scheduled_suspension_metric(code: &str, label: &str) -> ScheduledSuspensionMetric {
    ScheduledSuspensionMetric {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        version: 0,
        approved: false,
        retries: 0,
        status: ScheduledSuspensionMetricStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_suspension_metric("code-816", "label-207");
        assert_eq!(item.status, ScheduledSuspensionMetricStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_scheduled_suspension_metric("code-816", "label-207");
        if item.move_to(ScheduledSuspensionMetricStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSuspensionMetricStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_suspension_metric("code-816", "label-207");
        item.status = ScheduledSuspensionMetricStatus::Failed;
        assert!(
            item.move_to(ScheduledSuspensionMetricStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
