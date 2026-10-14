use super::*;

use acme_kit::text;

/// Where a default quote budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultQuoteBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultQuoteBudgetStatus {
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

impl std::fmt::Display for DefaultQuoteBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default quote budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQuoteBudget {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub floor_value: f64,
    pub external_ref: String,
    pub status: DefaultQuoteBudgetStatus,
}

impl DefaultQuoteBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultQuoteBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultQuoteBudgetStatus) -> bool {
        match self.status {
            DefaultQuoteBudgetStatus::New => {
                return next == DefaultQuoteBudgetStatus::Queued || next == DefaultQuoteBudgetStatus::Done;
            }
            DefaultQuoteBudgetStatus::Queued => {
                match next {
                    DefaultQuoteBudgetStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultQuoteBudgetStatus::Running => {
                match next {
                    DefaultQuoteBudgetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultQuoteBudgetStatus::Done => {
                match next {
                    DefaultQuoteBudgetStatus::Failed => {
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

    pub fn move_to(&mut self, next: DefaultQuoteBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default quote budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default quote budget", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.floor_value
    }

    pub fn copy(&self) -> DefaultQuoteBudget {
        DefaultQuoteBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            floor_value: self.floor_value,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a default quote budget in status new.
pub fn new_default_quote_budget(code: &str, title: &str) -> DefaultQuoteBudget {
    DefaultQuoteBudget {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        floor_value: 0.0,
        external_ref: "".to_string(),
        status: DefaultQuoteBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_quote_budget("code-505", "title-969");
        assert_eq!(item.status, DefaultQuoteBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_default_quote_budget("code-505", "title-969");
        if item.move_to(DefaultQuoteBudgetStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultQuoteBudgetStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_quote_budget("code-505", "title-969");
        item.status = DefaultQuoteBudgetStatus::Failed;
        assert!(
            item.move_to(DefaultQuoteBudgetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
