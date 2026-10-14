use super::*;

use acme_kit::text;

/// Where a limit ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LimitLedgerStatus {
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

impl std::fmt::Display for LimitLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a limit ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitLedger {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub score: f64,
    pub audience: String,
    pub grace_days: i64,
    pub limit: i64,
    pub cluster: String,
    pub batch_size: i64,
    pub status: LimitLedgerStatus,
}

impl LimitLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitLedgerStatus) -> bool {
        match self.status {
            LimitLedgerStatus::New => {
                return next == LimitLedgerStatus::Failed;
            }
            LimitLedgerStatus::Queued => {
                return next == LimitLedgerStatus::Running || next == LimitLedgerStatus::Done;
            }
            LimitLedgerStatus::Running => {
                return next == LimitLedgerStatus::Done || next == LimitLedgerStatus::Failed;
            }
            LimitLedgerStatus::Done => {
                match next {
                    LimitLedgerStatus::Failed => {
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

    pub fn move_to(&mut self, next: LimitLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.amount * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitLedger {
        LimitLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            score: self.score,
            audience: self.audience.clone(),
            grace_days: self.grace_days,
            limit: self.limit,
            cluster: self.cluster.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a limit ledger in status new.
pub fn new_limit_ledger(code: &str, title: &str) -> LimitLedger {
    LimitLedger {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        score: 0.0,
        audience: "".to_string(),
        grace_days: 0,
        limit: 0,
        cluster: "".to_string(),
        batch_size: 0,
        status: LimitLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_limit_ledger("code-521", "title-286");
        assert_eq!(item.status, LimitLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_limit_ledger("code-521", "title-286");
        if item.move_to(LimitLedgerStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitLedgerStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_limit_ledger("code-521", "title-286");
        item.status = LimitLedgerStatus::Failed;
        assert!(item.move_to(LimitLedgerStatus::New).is_err(), "expected move to fail");
    }
}
