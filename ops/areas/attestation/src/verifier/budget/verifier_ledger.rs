use super::*;

use acme_kit::text;

/// Where a verifier ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierLedgerStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl VerifierLedgerStatus {
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

impl std::fmt::Display for VerifierLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier ledger as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierLedger {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub quantity: i64,
    pub active: bool,
    pub port: i64,
    pub status: VerifierLedgerStatus,
}

impl VerifierLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierLedgerStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierLedgerStatus) -> bool {
        match self.status {
            VerifierLedgerStatus::Draft => {
                return next == VerifierLedgerStatus::Pending || next == VerifierLedgerStatus::Active;
            }
            VerifierLedgerStatus::Pending => {
                match next {
                    VerifierLedgerStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierLedgerStatus::Active => {
                match next {
                    VerifierLedgerStatus::Suspended | VerifierLedgerStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierLedgerStatus::Suspended => {
                return next == VerifierLedgerStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.port as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierLedger {
        VerifierLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            active: self.active,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a verifier ledger in status draft.
pub fn new_verifier_ledger(code: &str, label: &str) -> VerifierLedger {
    VerifierLedger {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        quantity: 0,
        active: false,
        port: 0,
        status: VerifierLedgerStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_verifier_ledger("code-975", "label-717");
        assert_eq!(item.status, VerifierLedgerStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_verifier_ledger("code-975", "label-717");
        if item.move_to(VerifierLedgerStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, VerifierLedgerStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_verifier_ledger("code-975", "label-717");
        item.status = VerifierLedgerStatus::Closed;
        assert!(
            item.move_to(VerifierLedgerStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
