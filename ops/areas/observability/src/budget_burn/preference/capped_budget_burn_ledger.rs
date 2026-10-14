use super::*;

use acme_kit::text;

/// Where a capped budget burn ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedBudgetBurnLedgerStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedBudgetBurnLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CappedBudgetBurnLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped budget burn ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedBudgetBurnLedger {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub namespace: String,
    pub checksum: String,
    pub tags: Vec<String>,
    pub status: CappedBudgetBurnLedgerStatus,
}

impl CappedBudgetBurnLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedBudgetBurnLedgerStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedBudgetBurnLedgerStatus) -> bool {
        match self.status {
            CappedBudgetBurnLedgerStatus::Draft => {
                return next == CappedBudgetBurnLedgerStatus::Suspended || next == CappedBudgetBurnLedgerStatus::Closed;
            }
            CappedBudgetBurnLedgerStatus::Pending => {
                match next {
                    CappedBudgetBurnLedgerStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBudgetBurnLedgerStatus::Active => {
                match next {
                    CappedBudgetBurnLedgerStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedBudgetBurnLedgerStatus::Suspended => {
                match next {
                    CappedBudgetBurnLedgerStatus::Closed => {
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

    pub fn move_to(&mut self, next: CappedBudgetBurnLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped budget burn ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> CappedBudgetBurnLedger {
        CappedBudgetBurnLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            namespace: self.namespace.clone(),
            checksum: self.checksum.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a capped budget burn ledger in status draft.
pub fn new_capped_budget_burn_ledger(code: &str, title: &str) -> CappedBudgetBurnLedger {
    CappedBudgetBurnLedger {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        namespace: "".to_string(),
        checksum: "".to_string(),
        tags: Vec::new(),
        status: CappedBudgetBurnLedgerStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_capped_budget_burn_ledger("code-718", "title-748");
        assert_eq!(item.status, CappedBudgetBurnLedgerStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_capped_budget_burn_ledger("code-718", "title-748");
        if item.move_to(CappedBudgetBurnLedgerStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedBudgetBurnLedgerStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_capped_budget_burn_ledger("code-718", "title-748");
        item.status = CappedBudgetBurnLedgerStatus::Closed;
        assert!(
            item.move_to(CappedBudgetBurnLedgerStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
