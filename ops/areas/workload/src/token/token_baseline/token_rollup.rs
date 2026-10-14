use super::*;

use acme_kit::text;

/// Where a token rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TokenRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TokenRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One token rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenRollup {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub notes: String,
    pub expires_at: i64,
    pub unit: String,
    pub locale: String,
    pub capacity: i64,
    pub checksum: String,
    pub summary: String,
    pub external_ref: String,
    pub status: TokenRollupStatus,
}

impl TokenRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenRollupStatus) -> bool {
        match self.status {
            TokenRollupStatus::Idle => {
                return next == TokenRollupStatus::Stopped;
            }
            TokenRollupStatus::Warming => {
                return next == TokenRollupStatus::Stopped;
            }
            TokenRollupStatus::Ready => {
                return next == TokenRollupStatus::Stopped;
            }
            TokenRollupStatus::Draining => {
                match next {
                    TokenRollupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TokenRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.capacity as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenRollup {
        TokenRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            notes: self.notes.clone(),
            expires_at: self.expires_at,
            unit: self.unit.clone(),
            locale: self.locale.clone(),
            capacity: self.capacity,
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a token rollup in status idle.
pub fn new_token_rollup(code: &str, label: &str) -> TokenRollup {
    TokenRollup {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        notes: "".to_string(),
        expires_at: 0,
        unit: "".to_string(),
        locale: "".to_string(),
        capacity: 0,
        checksum: "".to_string(),
        summary: "".to_string(),
        external_ref: "".to_string(),
        status: TokenRollupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_token_rollup("code-981", "label-788");
        assert_eq!(item.status, TokenRollupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_token_rollup("code-981", "label-788");
        if item.move_to(TokenRollupStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenRollupStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_token_rollup("code-981", "label-788");
        item.status = TokenRollupStatus::Stopped;
        assert!(item.move_to(TokenRollupStatus::Idle).is_err(), "expected move to fail");
    }
}
