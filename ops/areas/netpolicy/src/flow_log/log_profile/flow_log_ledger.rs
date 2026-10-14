use super::*;

use acme_kit::text;

/// Where a flow log ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FlowLogLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FlowLogLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FlowLogLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A flow log ledger as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogLedger {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub audience: String,
    pub owner_id: String,
    pub category: String,
    pub amount: f64,
    pub status: FlowLogLedgerStatus,
}

impl FlowLogLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FlowLogLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FlowLogLedgerStatus) -> bool {
        match self.status {
            FlowLogLedgerStatus::Planned => {
                match next {
                    FlowLogLedgerStatus::Scheduled | FlowLogLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogLedgerStatus::Scheduled => {
                return next == FlowLogLedgerStatus::InProgress || next == FlowLogLedgerStatus::Cancelled;
            }
            FlowLogLedgerStatus::InProgress => {
                match next {
                    FlowLogLedgerStatus::Completed | FlowLogLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FlowLogLedgerStatus::Completed => {
                match next {
                    FlowLogLedgerStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: FlowLogLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "flow log ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> FlowLogLedger {
        FlowLogLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            audience: self.audience.clone(),
            owner_id: self.owner_id.clone(),
            category: self.category.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a flow log ledger in status planned.
pub fn new_flow_log_ledger(code: &str, name: &str) -> FlowLogLedger {
    FlowLogLedger {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        audience: "".to_string(),
        owner_id: "".to_string(),
        category: "".to_string(),
        amount: 0.0,
        status: FlowLogLedgerStatus::Planned,
    }
}
