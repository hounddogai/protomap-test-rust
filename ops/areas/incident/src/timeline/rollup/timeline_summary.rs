use super::*;

use acme_kit::text;

/// Where a timeline summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineSummaryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TimelineSummaryStatus {
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

impl std::fmt::Display for TimelineSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSummary {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub visible: bool,
    pub notes: String,
    pub version: i64,
    pub attempts: i64,
    pub margin: f64,
    pub grace_days: i64,
    pub namespace: String,
    pub summary: String,
    pub status: TimelineSummaryStatus,
}

impl TimelineSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineSummaryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineSummaryStatus) -> bool {
        match self.status {
            TimelineSummaryStatus::New => {
                return next == TimelineSummaryStatus::Failed;
            }
            TimelineSummaryStatus::Queued => {
                return next == TimelineSummaryStatus::Running || next == TimelineSummaryStatus::Failed;
            }
            TimelineSummaryStatus::Running => {
                return next == TimelineSummaryStatus::Done || next == TimelineSummaryStatus::Failed;
            }
            TimelineSummaryStatus::Done => {
                return next == TimelineSummaryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.grace_days as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineSummary {
        TimelineSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            visible: self.visible,
            notes: self.notes.clone(),
            version: self.version,
            attempts: self.attempts,
            margin: self.margin,
            grace_days: self.grace_days,
            namespace: self.namespace.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a timeline summary in status new.
pub fn new_timeline_summary(code: &str, title: &str) -> TimelineSummary {
    TimelineSummary {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        visible: false,
        notes: "".to_string(),
        version: 0,
        attempts: 0,
        margin: 0.0,
        grace_days: 0,
        namespace: "".to_string(),
        summary: "".to_string(),
        status: TimelineSummaryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_timeline_summary("code-768", "title-419");
        assert_eq!(item.status, TimelineSummaryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_timeline_summary("code-768", "title-419");
        if item.move_to(TimelineSummaryStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineSummaryStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_timeline_summary("code-768", "title-419");
        item.status = TimelineSummaryStatus::Failed;
        assert!(
            item.move_to(TimelineSummaryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
