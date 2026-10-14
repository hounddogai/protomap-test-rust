use super::*;

use acme_kit::text;

/// Where a scheduled isolation ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledIsolationLedgerStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledIsolationLedgerStatus {
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

impl std::fmt::Display for ScheduledIsolationLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled isolation ledger as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationLedger {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub description: String,
    pub quantity: i64,
    pub timezone: String,
    pub priority: i64,
    pub score: f64,
    pub status: ScheduledIsolationLedgerStatus,
}

impl ScheduledIsolationLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledIsolationLedgerStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledIsolationLedgerStatus) -> bool {
        match self.status {
            ScheduledIsolationLedgerStatus::Idle => {
                match next {
                    ScheduledIsolationLedgerStatus::Ready | ScheduledIsolationLedgerStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledIsolationLedgerStatus::Warming => {
                match next {
                    ScheduledIsolationLedgerStatus::Ready | ScheduledIsolationLedgerStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledIsolationLedgerStatus::Ready => {
                return next == ScheduledIsolationLedgerStatus::Draining;
            }
            ScheduledIsolationLedgerStatus::Draining => {
                return next == ScheduledIsolationLedgerStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledIsolationLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled isolation ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled isolation ledger", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.created_at as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledIsolationLedger {
        ScheduledIsolationLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            description: self.description.clone(),
            quantity: self.quantity,
            timezone: self.timezone.clone(),
            priority: self.priority,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled isolation ledger in status idle.
pub fn new_scheduled_isolation_ledger(code: &str, name: &str) -> ScheduledIsolationLedger {
    ScheduledIsolationLedger {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        description: "".to_string(),
        quantity: 0,
        timezone: "".to_string(),
        priority: 0,
        score: 0.0,
        status: ScheduledIsolationLedgerStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_isolation_ledger("code-825", "name-819");
        assert_eq!(item.status, ScheduledIsolationLedgerStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_scheduled_isolation_ledger("code-825", "name-819");
        if item.move_to(ScheduledIsolationLedgerStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledIsolationLedgerStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_isolation_ledger("code-825", "name-819");
        item.status = ScheduledIsolationLedgerStatus::Stopped;
        assert!(
            item.move_to(ScheduledIsolationLedgerStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
