use super::*;

use acme_kit::text;

/// Where a scheduled isolation exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledIsolationExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledIsolationExceptionStatus {
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

impl std::fmt::Display for ScheduledIsolationExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled isolation exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationException {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub channel: String,
    pub active: bool,
    pub approved: bool,
    pub summary: String,
    pub timezone: String,
    pub status: ScheduledIsolationExceptionStatus,
}

impl ScheduledIsolationException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledIsolationExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledIsolationExceptionStatus) -> bool {
        match self.status {
            ScheduledIsolationExceptionStatus::Open => {
                match next {
                    ScheduledIsolationExceptionStatus::Reopened | ScheduledIsolationExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledIsolationExceptionStatus::InReview => {
                return next == ScheduledIsolationExceptionStatus::Resolved
                    || next == ScheduledIsolationExceptionStatus::Archived;
            }
            ScheduledIsolationExceptionStatus::Resolved => {
                match next {
                    ScheduledIsolationExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledIsolationExceptionStatus::Reopened => {
                return next == ScheduledIsolationExceptionStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledIsolationExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled isolation exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> ScheduledIsolationException {
        ScheduledIsolationException {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            channel: self.channel.clone(),
            active: self.active,
            approved: self.approved,
            summary: self.summary.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a scheduled isolation exception in status open.
pub fn new_scheduled_isolation_exception(code: &str, label: &str) -> ScheduledIsolationException {
    ScheduledIsolationException {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        channel: "".to_string(),
        active: false,
        approved: false,
        summary: "".to_string(),
        timezone: "".to_string(),
        status: ScheduledIsolationExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_isolation_exception("code-658", "label-904");
        assert_eq!(item.status, ScheduledIsolationExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_scheduled_isolation_exception("code-658", "label-904");
        if item.move_to(ScheduledIsolationExceptionStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledIsolationExceptionStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_isolation_exception("code-658", "label-904");
        item.status = ScheduledIsolationExceptionStatus::Archived;
        assert!(
            item.move_to(ScheduledIsolationExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
