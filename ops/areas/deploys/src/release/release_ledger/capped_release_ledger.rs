use super::*;

use acme_kit::text;

/// Where a capped release ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedReleaseLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedReleaseLedgerStatus {
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

impl std::fmt::Display for CappedReleaseLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped release ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedReleaseLedger {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub source: String,
    pub burst: i64,
    pub status: CappedReleaseLedgerStatus,
}

impl CappedReleaseLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedReleaseLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedReleaseLedgerStatus) -> bool {
        match self.status {
            CappedReleaseLedgerStatus::New => {
                return next == CappedReleaseLedgerStatus::Failed;
            }
            CappedReleaseLedgerStatus::Queued => {
                return next == CappedReleaseLedgerStatus::Done || next == CappedReleaseLedgerStatus::Failed;
            }
            CappedReleaseLedgerStatus::Running => {
                match next {
                    CappedReleaseLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedReleaseLedgerStatus::Done => {
                return next == CappedReleaseLedgerStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedReleaseLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped release ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        (self.maximum * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedReleaseLedger {
        CappedReleaseLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            source: self.source.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a capped release ledger in status new.
pub fn new_capped_release_ledger(code: &str, name: &str) -> CappedReleaseLedger {
    CappedReleaseLedger {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        source: "".to_string(),
        burst: 0,
        status: CappedReleaseLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_release_ledger("code-368", "name-409");
        assert_eq!(item.status, CappedReleaseLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_capped_release_ledger("code-368", "name-409");
        if item.move_to(CappedReleaseLedgerStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedReleaseLedgerStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_release_ledger("code-368", "name-409");
        item.status = CappedReleaseLedgerStatus::Failed;
        assert!(
            item.move_to(CappedReleaseLedgerStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
