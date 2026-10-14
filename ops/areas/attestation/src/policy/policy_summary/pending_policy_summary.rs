use super::*;

use acme_kit::text;

/// Where a pending policy summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPolicySummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PendingPolicySummaryStatus {
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

impl std::fmt::Display for PendingPolicySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending policy summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPolicySummary {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub contact_email: String,
    pub rotation_days: i64,
    pub archived: bool,
    pub quota_units: i64,
    pub retries: i64,
    pub status: PendingPolicySummaryStatus,
}

impl PendingPolicySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPolicySummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPolicySummaryStatus) -> bool {
        match self.status {
            PendingPolicySummaryStatus::Open => {
                match next {
                    PendingPolicySummaryStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPolicySummaryStatus::InReview => {
                match next {
                    PendingPolicySummaryStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPolicySummaryStatus::Resolved => {
                match next {
                    PendingPolicySummaryStatus::Reopened | PendingPolicySummaryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPolicySummaryStatus::Reopened => {
                return next == PendingPolicySummaryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingPolicySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending policy summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.quota_units as f64 - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPolicySummary {
        PendingPolicySummary {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            contact_email: self.contact_email.clone(),
            rotation_days: self.rotation_days,
            archived: self.archived,
            quota_units: self.quota_units,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending policy summary in status open.
pub fn new_pending_policy_summary(code: &str, name: &str) -> PendingPolicySummary {
    PendingPolicySummary {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        contact_email: "".to_string(),
        rotation_days: 0,
        archived: false,
        quota_units: 0,
        retries: 0,
        status: PendingPolicySummaryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_pending_policy_summary("code-864", "name-390");
        assert_eq!(item.status, PendingPolicySummaryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_pending_policy_summary("code-864", "name-390");
        if item.move_to(PendingPolicySummaryStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPolicySummaryStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_pending_policy_summary("code-864", "name-390");
        item.status = PendingPolicySummaryStatus::Archived;
        assert!(
            item.move_to(PendingPolicySummaryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
