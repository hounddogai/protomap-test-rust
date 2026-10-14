use super::*;

use acme_kit::text;

/// Where a token ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenLedgerStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TokenLedgerStatus {
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

impl std::fmt::Display for TokenLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenLedger {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub capacity: i64,
    pub ratio: f64,
    pub batch_size: i64,
    pub cluster: String,
    pub scope: String,
    pub status: TokenLedgerStatus,
}

impl TokenLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenLedgerStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenLedgerStatus) -> bool {
        match self.status {
            TokenLedgerStatus::Idle => {
                return next == TokenLedgerStatus::Stopped;
            }
            TokenLedgerStatus::Warming => {
                return next == TokenLedgerStatus::Draining || next == TokenLedgerStatus::Stopped;
            }
            TokenLedgerStatus::Ready => {
                match next {
                    TokenLedgerStatus::Draining | TokenLedgerStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenLedgerStatus::Draining => {
                return next == TokenLedgerStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "token ledger", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.capacity as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenLedger {
        TokenLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            capacity: self.capacity,
            ratio: self.ratio,
            batch_size: self.batch_size,
            cluster: self.cluster.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a token ledger in status idle.
pub fn new_token_ledger(code: &str, name: &str) -> TokenLedger {
    TokenLedger {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        capacity: 0,
        ratio: 0.0,
        batch_size: 0,
        cluster: "".to_string(),
        scope: "".to_string(),
        status: TokenLedgerStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_token_ledger("code-376", "name-262");
        assert_eq!(item.status, TokenLedgerStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_token_ledger("code-376", "name-262");
        if item.move_to(TokenLedgerStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenLedgerStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_token_ledger("code-376", "name-262");
        item.status = TokenLedgerStatus::Stopped;
        assert!(item.move_to(TokenLedgerStatus::Idle).is_err(), "expected move to fail");
    }
}
