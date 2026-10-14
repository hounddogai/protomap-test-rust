use super::*;

use acme_kit::text;

/// Where a token checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenCheckpointStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TokenCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TokenCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token checkpoint as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenCheckpoint {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub floor_value: f64,
    pub retries: i64,
    pub duration_minutes: i64,
    pub key_id: String,
    pub cipher: String,
    pub fingerprint: String,
    pub status: TokenCheckpointStatus,
}

impl TokenCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenCheckpointStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenCheckpointStatus) -> bool {
        match self.status {
            TokenCheckpointStatus::Proposed => {
                return next == TokenCheckpointStatus::Approved || next == TokenCheckpointStatus::Withdrawn;
            }
            TokenCheckpointStatus::Approved => {
                return next == TokenCheckpointStatus::Rejected;
            }
            TokenCheckpointStatus::Rejected => {
                return next == TokenCheckpointStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.floor_value - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenCheckpoint {
        TokenCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            floor_value: self.floor_value,
            retries: self.retries,
            duration_minutes: self.duration_minutes,
            key_id: self.key_id.clone(),
            cipher: self.cipher.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a token checkpoint in status proposed.
pub fn new_token_checkpoint(code: &str, title: &str) -> TokenCheckpoint {
    TokenCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        floor_value: 0.0,
        retries: 0,
        duration_minutes: 0,
        key_id: "".to_string(),
        cipher: "".to_string(),
        fingerprint: "".to_string(),
        status: TokenCheckpointStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_token_checkpoint("code-657", "title-105");
        assert_eq!(item.status, TokenCheckpointStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_token_checkpoint("code-657", "title-105");
        if item.move_to(TokenCheckpointStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenCheckpointStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_token_checkpoint("code-657", "title-105");
        item.status = TokenCheckpointStatus::Withdrawn;
        assert!(
            item.move_to(TokenCheckpointStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
