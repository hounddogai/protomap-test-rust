use super::*;

use acme_kit::text;

/// Where a timeline baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineBaselineStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TimelineBaselineStatus {
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

impl std::fmt::Display for TimelineBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBaseline {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub region: String,
    pub retries: i64,
    pub ceiling: f64,
    pub principal: String,
    pub status: TimelineBaselineStatus,
}

impl TimelineBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineBaselineStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineBaselineStatus) -> bool {
        match self.status {
            TimelineBaselineStatus::Planned => {
                match next {
                    TimelineBaselineStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineBaselineStatus::Scheduled => {
                match next {
                    TimelineBaselineStatus::InProgress | TimelineBaselineStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineBaselineStatus::InProgress => {
                return next == TimelineBaselineStatus::Completed;
            }
            TimelineBaselineStatus::Completed => {
                return next == TimelineBaselineStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "timeline baseline", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ceiling - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineBaseline {
        TimelineBaseline {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            region: self.region.clone(),
            retries: self.retries,
            ceiling: self.ceiling,
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a timeline baseline in status planned.
pub fn new_timeline_baseline(code: &str, name: &str) -> TimelineBaseline {
    TimelineBaseline {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        region: "".to_string(),
        retries: 0,
        ceiling: 0.0,
        principal: "".to_string(),
        status: TimelineBaselineStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_timeline_baseline("code-351", "name-310");
        assert_eq!(item.status, TimelineBaselineStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_timeline_baseline("code-351", "name-310");
        if item.move_to(TimelineBaselineStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineBaselineStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_timeline_baseline("code-351", "name-310");
        item.status = TimelineBaselineStatus::Cancelled;
        assert!(
            item.move_to(TimelineBaselineStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
