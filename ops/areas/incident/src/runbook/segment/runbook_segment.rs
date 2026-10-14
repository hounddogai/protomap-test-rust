use super::*;

use acme_kit::text;

/// Where a runbook segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RunbookSegmentStatus {
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

impl std::fmt::Display for RunbookSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSegment {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub issuer: String,
    pub archived: bool,
    pub batch_size: i64,
    pub budget: f64,
    pub cost: f64,
    pub window_days: i64,
    pub namespace: String,
    pub amount: f64,
    pub status: RunbookSegmentStatus,
}

impl RunbookSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookSegmentStatus) -> bool {
        match self.status {
            RunbookSegmentStatus::Open => {
                match next {
                    RunbookSegmentStatus::Reopened | RunbookSegmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookSegmentStatus::InReview => {
                return next == RunbookSegmentStatus::Archived;
            }
            RunbookSegmentStatus::Resolved => {
                return next == RunbookSegmentStatus::Reopened;
            }
            RunbookSegmentStatus::Reopened => {
                match next {
                    RunbookSegmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: RunbookSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.cost - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookSegment {
        RunbookSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            issuer: self.issuer.clone(),
            archived: self.archived,
            batch_size: self.batch_size,
            budget: self.budget,
            cost: self.cost,
            window_days: self.window_days,
            namespace: self.namespace.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a runbook segment in status open.
pub fn new_runbook_segment(code: &str, title: &str) -> RunbookSegment {
    RunbookSegment {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        issuer: "".to_string(),
        archived: false,
        batch_size: 0,
        budget: 0.0,
        cost: 0.0,
        window_days: 0,
        namespace: "".to_string(),
        amount: 0.0,
        status: RunbookSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_runbook_segment("code-858", "title-934");
        assert_eq!(item.status, RunbookSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_runbook_segment("code-858", "title-934");
        if item.move_to(RunbookSegmentStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookSegmentStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_runbook_segment("code-858", "title-934");
        item.status = RunbookSegmentStatus::Archived;
        assert!(
            item.move_to(RunbookSegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
