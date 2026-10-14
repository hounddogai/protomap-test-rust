use super::*;

use acme_kit::text;

/// Where a redaction summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RedactionSummaryStatus {
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

impl std::fmt::Display for RedactionSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionSummary {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub priority: i64,
    pub ends_at: i64,
    pub ttl_seconds: i64,
    pub revision: i64,
    pub amount: f64,
    pub minimum: f64,
    pub status: RedactionSummaryStatus,
}

impl RedactionSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionSummaryStatus) -> bool {
        match self.status {
            RedactionSummaryStatus::Open => {
                return next == RedactionSummaryStatus::InReview;
            }
            RedactionSummaryStatus::InReview => {
                match next {
                    RedactionSummaryStatus::Resolved | RedactionSummaryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionSummaryStatus::Resolved => {
                match next {
                    RedactionSummaryStatus::Reopened | RedactionSummaryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionSummaryStatus::Reopened => {
                match next {
                    RedactionSummaryStatus::Archived => {
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

    pub fn move_to(&mut self, next: RedactionSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.priority as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionSummary {
        RedactionSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            priority: self.priority,
            ends_at: self.ends_at,
            ttl_seconds: self.ttl_seconds,
            revision: self.revision,
            amount: self.amount,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a redaction summary in status open.
pub fn new_redaction_summary(code: &str, title: &str) -> RedactionSummary {
    RedactionSummary {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        priority: 0,
        ends_at: 0,
        ttl_seconds: 0,
        revision: 0,
        amount: 0.0,
        minimum: 0.0,
        status: RedactionSummaryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_redaction_summary("code-553", "title-455");
        assert_eq!(item.status, RedactionSummaryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_redaction_summary("code-553", "title-455");
        if item.move_to(RedactionSummaryStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionSummaryStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_redaction_summary("code-553", "title-455");
        item.status = RedactionSummaryStatus::Archived;
        assert!(
            item.move_to(RedactionSummaryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
