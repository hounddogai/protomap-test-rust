use super::*;

use acme_kit::text;

/// Where a lenient action item ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientActionItemLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientActionItemLedgerStatus {
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

impl std::fmt::Display for LenientActionItemLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient action item ledger as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemLedger {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub port: i64,
    pub threshold: f64,
    pub scope: String,
    pub external_ref: String,
    pub amount: f64,
    pub revision: i64,
    pub issuer: String,
    pub region: String,
    pub status: LenientActionItemLedgerStatus,
}

impl LenientActionItemLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientActionItemLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientActionItemLedgerStatus) -> bool {
        match self.status {
            LenientActionItemLedgerStatus::New => {
                return next == LenientActionItemLedgerStatus::Queued || next == LenientActionItemLedgerStatus::Running;
            }
            LenientActionItemLedgerStatus::Queued => {
                match next {
                    LenientActionItemLedgerStatus::Running | LenientActionItemLedgerStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientActionItemLedgerStatus::Running => {
                return next == LenientActionItemLedgerStatus::Done || next == LenientActionItemLedgerStatus::Failed;
            }
            LenientActionItemLedgerStatus::Done => {
                return next == LenientActionItemLedgerStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientActionItemLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient action item ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.revision as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientActionItemLedger {
        LenientActionItemLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            port: self.port,
            threshold: self.threshold,
            scope: self.scope.clone(),
            external_ref: self.external_ref.clone(),
            amount: self.amount,
            revision: self.revision,
            issuer: self.issuer.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient action item ledger in status new.
pub fn new_lenient_action_item_ledger(code: &str, title: &str) -> LenientActionItemLedger {
    LenientActionItemLedger {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        port: 0,
        threshold: 0.0,
        scope: "".to_string(),
        external_ref: "".to_string(),
        amount: 0.0,
        revision: 0,
        issuer: "".to_string(),
        region: "".to_string(),
        status: LenientActionItemLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_action_item_ledger("code-595", "title-643");
        assert_eq!(item.status, LenientActionItemLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_lenient_action_item_ledger("code-595", "title-643");
        if item.move_to(LenientActionItemLedgerStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientActionItemLedgerStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_action_item_ledger("code-595", "title-643");
        item.status = LenientActionItemLedgerStatus::Failed;
        assert!(
            item.move_to(LenientActionItemLedgerStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
