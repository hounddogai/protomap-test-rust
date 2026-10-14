use super::*;

use acme_kit::text;

/// Where a scheduled failover summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFailoverSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledFailoverSummaryStatus {
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

impl std::fmt::Display for ScheduledFailoverSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled failover summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSummary {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub priority: i64,
    pub contact_email: String,
    pub archived: bool,
    pub reviewer: String,
    pub status: ScheduledFailoverSummaryStatus,
}

impl ScheduledFailoverSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFailoverSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFailoverSummaryStatus) -> bool {
        match self.status {
            ScheduledFailoverSummaryStatus::Open => {
                match next {
                    ScheduledFailoverSummaryStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFailoverSummaryStatus::InReview => {
                match next {
                    ScheduledFailoverSummaryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFailoverSummaryStatus::Resolved => {
                return next == ScheduledFailoverSummaryStatus::Reopened
                    || next == ScheduledFailoverSummaryStatus::Archived;
            }
            ScheduledFailoverSummaryStatus::Reopened => {
                match next {
                    ScheduledFailoverSummaryStatus::Archived => {
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

    pub fn move_to(&mut self, next: ScheduledFailoverSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled failover summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.priority as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFailoverSummary {
        ScheduledFailoverSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            priority: self.priority,
            contact_email: self.contact_email.clone(),
            archived: self.archived,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled failover summary in status open.
pub fn new_scheduled_failover_summary(code: &str, name: &str) -> ScheduledFailoverSummary {
    ScheduledFailoverSummary {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        priority: 0,
        contact_email: "".to_string(),
        archived: false,
        reviewer: "".to_string(),
        status: ScheduledFailoverSummaryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_failover_summary("code-608", "name-453");
        assert_eq!(item.status, ScheduledFailoverSummaryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_scheduled_failover_summary("code-608", "name-453");
        if item.move_to(ScheduledFailoverSummaryStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFailoverSummaryStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_failover_summary("code-608", "name-453");
        item.status = ScheduledFailoverSummaryStatus::Archived;
        assert!(
            item.move_to(ScheduledFailoverSummaryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
