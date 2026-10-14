use super::*;

use acme_kit::text;

/// Where a redaction budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RedactionBudgetStatus {
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

impl std::fmt::Display for RedactionBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction budget as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionBudget {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub priority: i64,
    pub revision: i64,
    pub segment: String,
    pub rate: f64,
    pub external_ref: String,
    pub burst: i64,
    pub notes: String,
    pub status: RedactionBudgetStatus,
}

impl RedactionBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionBudgetStatus) -> bool {
        match self.status {
            RedactionBudgetStatus::New => {
                return next == RedactionBudgetStatus::Done || next == RedactionBudgetStatus::Failed;
            }
            RedactionBudgetStatus::Queued => {
                match next {
                    RedactionBudgetStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionBudgetStatus::Running => {
                return next == RedactionBudgetStatus::Done;
            }
            RedactionBudgetStatus::Done => {
                match next {
                    RedactionBudgetStatus::Failed => {
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

    pub fn move_to(&mut self, next: RedactionBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "redaction budget", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionBudget {
        RedactionBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            priority: self.priority,
            revision: self.revision,
            segment: self.segment.clone(),
            rate: self.rate,
            external_ref: self.external_ref.clone(),
            burst: self.burst,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a redaction budget in status new.
pub fn new_redaction_budget(code: &str, name: &str) -> RedactionBudget {
    RedactionBudget {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        priority: 0,
        revision: 0,
        segment: "".to_string(),
        rate: 0.0,
        external_ref: "".to_string(),
        burst: 0,
        notes: "".to_string(),
        status: RedactionBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_redaction_budget("code-945", "name-81");
        assert_eq!(item.status, RedactionBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_redaction_budget("code-945", "name-81");
        if item.move_to(RedactionBudgetStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionBudgetStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_redaction_budget("code-945", "name-81");
        item.status = RedactionBudgetStatus::Failed;
        assert!(
            item.move_to(RedactionBudgetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
