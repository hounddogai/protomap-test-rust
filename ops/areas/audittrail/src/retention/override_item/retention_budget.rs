use super::*;

use acme_kit::text;

/// Where a retention budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionBudgetStatus {
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

impl std::fmt::Display for RetentionBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A retention budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBudget {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub label: String,
    pub region: String,
    pub duration_minutes: i64,
    pub archived: bool,
    pub status: RetentionBudgetStatus,
}

impl RetentionBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionBudgetStatus) -> bool {
        match self.status {
            RetentionBudgetStatus::New => {
                match next {
                    RetentionBudgetStatus::Running | RetentionBudgetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionBudgetStatus::Queued => {
                match next {
                    RetentionBudgetStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionBudgetStatus::Running => {
                return next == RetentionBudgetStatus::Done;
            }
            RetentionBudgetStatus::Done => {
                return next == RetentionBudgetStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> RetentionBudget {
        RetentionBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            duration_minutes: self.duration_minutes,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a retention budget in status new.
pub fn new_retention_budget(code: &str, name: &str) -> RetentionBudget {
    RetentionBudget {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        label: "".to_string(),
        region: "".to_string(),
        duration_minutes: 0,
        archived: false,
        status: RetentionBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_retention_budget("code-530", "name-967");
        assert_eq!(item.status, RetentionBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_retention_budget("code-530", "name-967");
        if item.move_to(RetentionBudgetStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionBudgetStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_retention_budget("code-530", "name-967");
        item.status = RetentionBudgetStatus::Failed;
        assert!(
            item.move_to(RetentionBudgetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
