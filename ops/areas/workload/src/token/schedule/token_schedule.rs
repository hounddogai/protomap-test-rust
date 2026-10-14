use super::*;

use acme_kit::text;

/// Where a token schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TokenScheduleStatus {
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

impl std::fmt::Display for TokenScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One token schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSchedule {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub scope: String,
    pub discount: f64,
    pub attempts: i64,
    pub budget: f64,
    pub key_id: String,
    pub retries: i64,
    pub status: TokenScheduleStatus,
}

impl TokenSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenScheduleStatus) -> bool {
        match self.status {
            TokenScheduleStatus::New => {
                return next == TokenScheduleStatus::Failed;
            }
            TokenScheduleStatus::Queued => {
                return next == TokenScheduleStatus::Failed;
            }
            TokenScheduleStatus::Running => {
                match next {
                    TokenScheduleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenScheduleStatus::Done => {
                match next {
                    TokenScheduleStatus::Failed => {
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

    pub fn move_to(&mut self, next: TokenScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.retries as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenSchedule {
        TokenSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            scope: self.scope.clone(),
            discount: self.discount,
            attempts: self.attempts,
            budget: self.budget,
            key_id: self.key_id.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a token schedule in status new.
pub fn new_token_schedule(code: &str, label: &str) -> TokenSchedule {
    TokenSchedule {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        scope: "".to_string(),
        discount: 0.0,
        attempts: 0,
        budget: 0.0,
        key_id: "".to_string(),
        retries: 0,
        status: TokenScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_token_schedule("code-191", "label-75");
        assert_eq!(item.status, TokenScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_token_schedule("code-191", "label-75");
        if item.move_to(TokenScheduleStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenScheduleStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_token_schedule("code-191", "label-75");
        item.status = TokenScheduleStatus::Failed;
        assert!(item.move_to(TokenScheduleStatus::New).is_err(), "expected move to fail");
    }
}
