use super::*;

use acme_kit::text;

/// Where a fallback query milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackQueryMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackQueryMilestoneStatus {
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

impl std::fmt::Display for FallbackQueryMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback query milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackQueryMilestone {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub quota_units: i64,
    pub namespace: String,
    pub issuer: String,
    pub retries: i64,
    pub status: FallbackQueryMilestoneStatus,
}

impl FallbackQueryMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackQueryMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackQueryMilestoneStatus) -> bool {
        match self.status {
            FallbackQueryMilestoneStatus::New => {
                return next == FallbackQueryMilestoneStatus::Running;
            }
            FallbackQueryMilestoneStatus::Queued => {
                match next {
                    FallbackQueryMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackQueryMilestoneStatus::Running => {
                return next == FallbackQueryMilestoneStatus::Done || next == FallbackQueryMilestoneStatus::Failed;
            }
            FallbackQueryMilestoneStatus::Done => {
                match next {
                    FallbackQueryMilestoneStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackQueryMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback query milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.quota_units as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackQueryMilestone {
        FallbackQueryMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            quota_units: self.quota_units,
            namespace: self.namespace.clone(),
            issuer: self.issuer.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback query milestone in status new.
pub fn new_fallback_query_milestone(code: &str, title: &str) -> FallbackQueryMilestone {
    FallbackQueryMilestone {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        quota_units: 0,
        namespace: "".to_string(),
        issuer: "".to_string(),
        retries: 0,
        status: FallbackQueryMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_query_milestone("code-547", "title-8");
        assert_eq!(item.status, FallbackQueryMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_fallback_query_milestone("code-547", "title-8");
        if item.move_to(FallbackQueryMilestoneStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackQueryMilestoneStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_query_milestone("code-547", "title-8");
        item.status = FallbackQueryMilestoneStatus::Failed;
        assert!(
            item.move_to(FallbackQueryMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
