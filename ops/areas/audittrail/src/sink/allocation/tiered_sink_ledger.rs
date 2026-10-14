use super::*;

use acme_kit::text;

/// Where a tiered sink ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSinkLedgerStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredSinkLedgerStatus {
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

impl std::fmt::Display for TieredSinkLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered sink ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkLedger {
    pub code: String,
    pub title: String,
    pub region: String,
    pub active: bool,
    pub fingerprint: String,
    pub quantity: i64,
    pub status: TieredSinkLedgerStatus,
}

impl TieredSinkLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSinkLedgerStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSinkLedgerStatus) -> bool {
        match self.status {
            TieredSinkLedgerStatus::Draft => {
                match next {
                    TieredSinkLedgerStatus::Active | TieredSinkLedgerStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSinkLedgerStatus::Pending => {
                match next {
                    TieredSinkLedgerStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSinkLedgerStatus::Active => {
                return next == TieredSinkLedgerStatus::Suspended;
            }
            TieredSinkLedgerStatus::Suspended => {
                return next == TieredSinkLedgerStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredSinkLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered sink ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> TieredSinkLedger {
        TieredSinkLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            active: self.active,
            fingerprint: self.fingerprint.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a tiered sink ledger in status draft.
pub fn new_tiered_sink_ledger(code: &str, title: &str) -> TieredSinkLedger {
    TieredSinkLedger {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        active: false,
        fingerprint: "".to_string(),
        quantity: 0,
        status: TieredSinkLedgerStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_sink_ledger("code-588", "title-190");
        assert_eq!(item.status, TieredSinkLedgerStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_tiered_sink_ledger("code-588", "title-190");
        if item.move_to(TieredSinkLedgerStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredSinkLedgerStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_sink_ledger("code-588", "title-190");
        item.status = TieredSinkLedgerStatus::Closed;
        assert!(
            item.move_to(TieredSinkLedgerStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
