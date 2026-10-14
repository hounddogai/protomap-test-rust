use super::*;

use acme_kit::text;

/// Where a token milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenMilestoneStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TokenMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for TokenMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One token milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenMilestone {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub visible: bool,
    pub audience: String,
    pub status: TokenMilestoneStatus,
}

impl TokenMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenMilestoneStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenMilestoneStatus) -> bool {
        match self.status {
            TokenMilestoneStatus::Planned => {
                return next == TokenMilestoneStatus::Cancelled;
            }
            TokenMilestoneStatus::Scheduled => {
                match next {
                    TokenMilestoneStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenMilestoneStatus::InProgress => {
                match next {
                    TokenMilestoneStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenMilestoneStatus::Completed => {
                return next == TokenMilestoneStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.window_days as f64
    }

    pub fn copy(&self) -> TokenMilestone {
        TokenMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            visible: self.visible,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a token milestone in status planned.
pub fn new_token_milestone(code: &str, name: &str) -> TokenMilestone {
    TokenMilestone {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        visible: false,
        audience: "".to_string(),
        status: TokenMilestoneStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_token_milestone("code-500", "name-849");
        assert_eq!(item.status, TokenMilestoneStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_token_milestone("code-500", "name-849");
        if item.move_to(TokenMilestoneStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenMilestoneStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_token_milestone("code-500", "name-849");
        item.status = TokenMilestoneStatus::Cancelled;
        assert!(
            item.move_to(TokenMilestoneStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
