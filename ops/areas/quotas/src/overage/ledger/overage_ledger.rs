use super::*;

use acme_kit::text;

/// Where a overage ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl OverageLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for OverageLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage ledger as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageLedger {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub reviewer: String,
    pub audience: String,
    pub visible: bool,
    pub principal: String,
    pub namespace: String,
    pub status: OverageLedgerStatus,
}

impl OverageLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageLedgerStatus) -> bool {
        match self.status {
            OverageLedgerStatus::Proposed => {
                return next == OverageLedgerStatus::Rejected;
            }
            OverageLedgerStatus::Approved => {
                match next {
                    OverageLedgerStatus::Rejected | OverageLedgerStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageLedgerStatus::Rejected => {
                return next == OverageLedgerStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage ledger {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> OverageLedger {
        OverageLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            reviewer: self.reviewer.clone(),
            audience: self.audience.clone(),
            visible: self.visible,
            principal: self.principal.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a overage ledger in status proposed.
pub fn new_overage_ledger(code: &str, label: &str) -> OverageLedger {
    OverageLedger {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        reviewer: "".to_string(),
        audience: "".to_string(),
        visible: false,
        principal: "".to_string(),
        namespace: "".to_string(),
        status: OverageLedgerStatus::Proposed,
    }
}
