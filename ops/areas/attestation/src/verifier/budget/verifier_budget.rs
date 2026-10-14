use super::*;

use acme_kit::text;

/// Where a verifier budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl VerifierBudgetStatus {
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

impl std::fmt::Display for VerifierBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBudget {
    pub code: String,
    pub name: String,
    pub host: String,
    pub timezone: String,
    pub ratio: f64,
    pub status: VerifierBudgetStatus,
}

impl VerifierBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierBudgetStatus) -> bool {
        match self.status {
            VerifierBudgetStatus::New => {
                return next == VerifierBudgetStatus::Done;
            }
            VerifierBudgetStatus::Queued => {
                return next == VerifierBudgetStatus::Done;
            }
            VerifierBudgetStatus::Running => {
                match next {
                    VerifierBudgetStatus::Done | VerifierBudgetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierBudgetStatus::Done => {
                return next == VerifierBudgetStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> VerifierBudget {
        VerifierBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            timezone: self.timezone.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a verifier budget in status new.
pub fn new_verifier_budget(code: &str, name: &str) -> VerifierBudget {
    VerifierBudget {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        timezone: "".to_string(),
        ratio: 0.0,
        status: VerifierBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_verifier_budget("code-745", "name-72");
        assert_eq!(item.status, VerifierBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_verifier_budget("code-745", "name-72");
        if item.move_to(VerifierBudgetStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierBudgetStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_verifier_budget("code-745", "name-72");
        item.status = VerifierBudgetStatus::Failed;
        assert!(
            item.move_to(VerifierBudgetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
