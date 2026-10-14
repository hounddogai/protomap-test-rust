use super::*;

use acme_kit::text;

/// Where a nonce ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl NonceLedgerStatus {
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

impl std::fmt::Display for NonceLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce ledger as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceLedger {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub ratio: f64,
    pub expires_at: i64,
    pub active: bool,
    pub retries: i64,
    pub rate: f64,
    pub cost: f64,
    pub principal: String,
    pub approved: bool,
    pub status: NonceLedgerStatus,
}

impl NonceLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceLedgerStatus) -> bool {
        match self.status {
            NonceLedgerStatus::New => {
                match next {
                    NonceLedgerStatus::Running | NonceLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceLedgerStatus::Queued => {
                match next {
                    NonceLedgerStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceLedgerStatus::Running => {
                match next {
                    NonceLedgerStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceLedgerStatus::Done => {
                match next {
                    NonceLedgerStatus::Failed => {
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

    pub fn move_to(&mut self, next: NonceLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce ledger {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.cost - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceLedger {
        NonceLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            ratio: self.ratio,
            expires_at: self.expires_at,
            active: self.active,
            retries: self.retries,
            rate: self.rate,
            cost: self.cost,
            principal: self.principal.clone(),
            approved: self.approved,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a nonce ledger in status new.
pub fn new_nonce_ledger(code: &str, title: &str) -> NonceLedger {
    NonceLedger {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        ratio: 0.0,
        expires_at: 0,
        active: false,
        retries: 0,
        rate: 0.0,
        cost: 0.0,
        principal: "".to_string(),
        approved: false,
        status: NonceLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_nonce_ledger("code-712", "title-498");
        assert_eq!(item.status, NonceLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_nonce_ledger("code-712", "title-498");
        if item.move_to(NonceLedgerStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceLedgerStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_nonce_ledger("code-712", "title-498");
        item.status = NonceLedgerStatus::Failed;
        assert!(item.move_to(NonceLedgerStatus::New).is_err(), "expected move to fail");
    }
}
