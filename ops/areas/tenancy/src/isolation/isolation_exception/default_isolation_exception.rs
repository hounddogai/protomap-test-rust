use super::*;

use acme_kit::text;

/// Where a default isolation exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultIsolationExceptionStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultIsolationExceptionStatus {
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

impl std::fmt::Display for DefaultIsolationExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default isolation exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIsolationException {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub attempts: i64,
    pub segment: String,
    pub amount: f64,
    pub status: DefaultIsolationExceptionStatus,
}

impl DefaultIsolationException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultIsolationExceptionStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultIsolationExceptionStatus) -> bool {
        match self.status {
            DefaultIsolationExceptionStatus::Planned => {
                return next == DefaultIsolationExceptionStatus::InProgress;
            }
            DefaultIsolationExceptionStatus::Scheduled => {
                return next == DefaultIsolationExceptionStatus::InProgress
                    || next == DefaultIsolationExceptionStatus::Completed;
            }
            DefaultIsolationExceptionStatus::InProgress => {
                return next == DefaultIsolationExceptionStatus::Cancelled;
            }
            DefaultIsolationExceptionStatus::Completed => {
                return next == DefaultIsolationExceptionStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultIsolationExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default isolation exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default isolation exception", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.priority as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultIsolationException {
        DefaultIsolationException {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            attempts: self.attempts,
            segment: self.segment.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a default isolation exception in status planned.
pub fn new_default_isolation_exception(code: &str, title: &str) -> DefaultIsolationException {
    DefaultIsolationException {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        attempts: 0,
        segment: "".to_string(),
        amount: 0.0,
        status: DefaultIsolationExceptionStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_isolation_exception("code-215", "title-476");
        assert_eq!(item.status, DefaultIsolationExceptionStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_default_isolation_exception("code-215", "title-476");
        if item.move_to(DefaultIsolationExceptionStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultIsolationExceptionStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_isolation_exception("code-215", "title-476");
        item.status = DefaultIsolationExceptionStatus::Cancelled;
        assert!(
            item.move_to(DefaultIsolationExceptionStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
