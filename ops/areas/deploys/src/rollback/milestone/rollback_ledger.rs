use super::*;

use acme_kit::text;

/// Where a rollback ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackLedgerStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollbackLedgerStatus {
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

impl std::fmt::Display for RollbackLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackLedger {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub principal: String,
    pub summary: String,
    pub namespace: String,
    pub ratio: f64,
    pub label: String,
    pub host: String,
    pub version: i64,
    pub cluster: String,
    pub status: RollbackLedgerStatus,
}

impl RollbackLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackLedgerStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackLedgerStatus) -> bool {
        match self.status {
            RollbackLedgerStatus::Idle => {
                match next {
                    RollbackLedgerStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackLedgerStatus::Warming => {
                return next == RollbackLedgerStatus::Ready || next == RollbackLedgerStatus::Draining;
            }
            RollbackLedgerStatus::Ready => {
                match next {
                    RollbackLedgerStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackLedgerStatus::Draining => {
                return next == RollbackLedgerStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.version as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackLedger {
        RollbackLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            principal: self.principal.clone(),
            summary: self.summary.clone(),
            namespace: self.namespace.clone(),
            ratio: self.ratio,
            label: self.label.clone(),
            host: self.host.clone(),
            version: self.version,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rollback ledger in status idle.
pub fn new_rollback_ledger(code: &str, title: &str) -> RollbackLedger {
    RollbackLedger {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        principal: "".to_string(),
        summary: "".to_string(),
        namespace: "".to_string(),
        ratio: 0.0,
        label: "".to_string(),
        host: "".to_string(),
        version: 0,
        cluster: "".to_string(),
        status: RollbackLedgerStatus::Idle,
    }
}
