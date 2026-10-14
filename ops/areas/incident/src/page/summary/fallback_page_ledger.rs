use super::*;

use acme_kit::text;

/// Where a fallback page ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPageLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackPageLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackPageLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback page ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageLedger {
    pub code: String,
    pub name: String,
    pub country: String,
    pub rotation_days: i64,
    pub cluster: String,
    pub key_id: String,
    pub status: FallbackPageLedgerStatus,
}

impl FallbackPageLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPageLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPageLedgerStatus) -> bool {
        match self.status {
            FallbackPageLedgerStatus::Created => {
                return next == FallbackPageLedgerStatus::Validated;
            }
            FallbackPageLedgerStatus::Validated => {
                match next {
                    FallbackPageLedgerStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPageLedgerStatus::Published => {
                return next == FallbackPageLedgerStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPageLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback page ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> FallbackPageLedger {
        FallbackPageLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            rotation_days: self.rotation_days,
            cluster: self.cluster.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a fallback page ledger in status created.
pub fn new_fallback_page_ledger(code: &str, name: &str) -> FallbackPageLedger {
    FallbackPageLedger {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        rotation_days: 0,
        cluster: "".to_string(),
        key_id: "".to_string(),
        status: FallbackPageLedgerStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_page_ledger("code-316", "name-23");
        assert_eq!(item.status, FallbackPageLedgerStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_fallback_page_ledger("code-316", "name-23");
        if item.move_to(FallbackPageLedgerStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPageLedgerStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_page_ledger("code-316", "name-23");
        item.status = FallbackPageLedgerStatus::Retired;
        assert!(
            item.move_to(FallbackPageLedgerStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
