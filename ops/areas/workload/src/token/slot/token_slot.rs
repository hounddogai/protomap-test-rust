use super::*;

use acme_kit::text;

/// Where a token slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TokenSlotStatus {
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

impl std::fmt::Display for TokenSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token slot as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenSlot {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub window_days: i64,
    pub notes: String,
    pub status: TokenSlotStatus,
}

impl TokenSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenSlotStatus) -> bool {
        match self.status {
            TokenSlotStatus::New => {
                match next {
                    TokenSlotStatus::Queued | TokenSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenSlotStatus::Queued => {
                return next == TokenSlotStatus::Failed;
            }
            TokenSlotStatus::Running => {
                match next {
                    TokenSlotStatus::Done | TokenSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenSlotStatus::Done => {
                match next {
                    TokenSlotStatus::Failed => {
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

    pub fn move_to(&mut self, next: TokenSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "token slot", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenSlot {
        TokenSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            window_days: self.window_days,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a token slot in status new.
pub fn new_token_slot(code: &str, label: &str) -> TokenSlot {
    TokenSlot {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        window_days: 0,
        notes: "".to_string(),
        status: TokenSlotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_token_slot("code-343", "label-968");
        assert_eq!(item.status, TokenSlotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_token_slot("code-343", "label-968");
        if item.move_to(TokenSlotStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenSlotStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_token_slot("code-343", "label-968");
        item.status = TokenSlotStatus::Failed;
        assert!(item.move_to(TokenSlotStatus::New).is_err(), "expected move to fail");
    }
}
