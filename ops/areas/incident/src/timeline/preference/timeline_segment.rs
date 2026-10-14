use super::*;

use acme_kit::text;

/// Where a timeline segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TimelineSegmentStatus {
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

impl std::fmt::Display for TimelineSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineSegment {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub version: i64,
    pub margin: f64,
    pub discount: f64,
    pub status: TimelineSegmentStatus,
}

impl TimelineSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineSegmentStatus) -> bool {
        match self.status {
            TimelineSegmentStatus::Planned => {
                match next {
                    TimelineSegmentStatus::InProgress | TimelineSegmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineSegmentStatus::Scheduled => {
                match next {
                    TimelineSegmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineSegmentStatus::InProgress => {
                return next == TimelineSegmentStatus::Cancelled;
            }
            TimelineSegmentStatus::Completed => {
                match next {
                    TimelineSegmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TimelineSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "timeline segment", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.batch_size as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineSegment {
        TimelineSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            version: self.version,
            margin: self.margin,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a timeline segment in status planned.
pub fn new_timeline_segment(code: &str, name: &str) -> TimelineSegment {
    TimelineSegment {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        version: 0,
        margin: 0.0,
        discount: 0.0,
        status: TimelineSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_timeline_segment("code-391", "name-438");
        assert_eq!(item.status, TimelineSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_timeline_segment("code-391", "name-438");
        if item.move_to(TimelineSegmentStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineSegmentStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_timeline_segment("code-391", "name-438");
        item.status = TimelineSegmentStatus::Cancelled;
        assert!(
            item.move_to(TimelineSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
