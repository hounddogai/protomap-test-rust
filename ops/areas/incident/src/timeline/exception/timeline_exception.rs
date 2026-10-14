use super::*;

use acme_kit::text;

/// Where a timeline exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TimelineExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TimelineExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One timeline exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineException {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub country: String,
    pub capacity: i64,
    pub sequence: i64,
    pub status: TimelineExceptionStatus,
}

impl TimelineException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineExceptionStatus) -> bool {
        match self.status {
            TimelineExceptionStatus::Idle => {
                return next == TimelineExceptionStatus::Ready;
            }
            TimelineExceptionStatus::Warming => {
                return next == TimelineExceptionStatus::Draining;
            }
            TimelineExceptionStatus::Ready => {
                match next {
                    TimelineExceptionStatus::Draining | TimelineExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineExceptionStatus::Draining => {
                match next {
                    TimelineExceptionStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TimelineExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.capacity as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineException {
        TimelineException {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            country: self.country.clone(),
            capacity: self.capacity,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a timeline exception in status idle.
pub fn new_timeline_exception(code: &str, title: &str) -> TimelineException {
    TimelineException {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        country: "".to_string(),
        capacity: 0,
        sequence: 0,
        status: TimelineExceptionStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_timeline_exception("code-485", "title-44");
        assert_eq!(item.status, TimelineExceptionStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_timeline_exception("code-485", "title-44");
        if item.move_to(TimelineExceptionStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineExceptionStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_timeline_exception("code-485", "title-44");
        item.status = TimelineExceptionStatus::Stopped;
        assert!(
            item.move_to(TimelineExceptionStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
