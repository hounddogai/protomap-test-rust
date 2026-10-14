use super::*;

use acme_kit::text;

/// Where a dashboard ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardLedgerStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DashboardLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for DashboardLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dashboard ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardLedger {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub active: bool,
    pub budget: f64,
    pub cipher: String,
    pub amount: f64,
    pub capacity: i64,
    pub status: DashboardLedgerStatus,
}

impl DashboardLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardLedgerStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardLedgerStatus) -> bool {
        match self.status {
            DashboardLedgerStatus::Idle => {
                return next == DashboardLedgerStatus::Warming || next == DashboardLedgerStatus::Ready;
            }
            DashboardLedgerStatus::Warming => {
                match next {
                    DashboardLedgerStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardLedgerStatus::Ready => {
                match next {
                    DashboardLedgerStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardLedgerStatus::Draining => {
                return next == DashboardLedgerStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dashboard ledger", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.limit as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardLedger {
        DashboardLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            active: self.active,
            budget: self.budget,
            cipher: self.cipher.clone(),
            amount: self.amount,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dashboard ledger in status idle.
pub fn new_dashboard_ledger(code: &str, name: &str) -> DashboardLedger {
    DashboardLedger {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        active: false,
        budget: 0.0,
        cipher: "".to_string(),
        amount: 0.0,
        capacity: 0,
        status: DashboardLedgerStatus::Idle,
    }
}
