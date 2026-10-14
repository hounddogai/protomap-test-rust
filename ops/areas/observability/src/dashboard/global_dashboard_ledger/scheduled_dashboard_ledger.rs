use super::*;

use acme_kit::text;

/// Where a scheduled dashboard ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDashboardLedgerStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledDashboardLedgerStatus {
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

impl std::fmt::Display for ScheduledDashboardLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled dashboard ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardLedger {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub version: i64,
    pub expires_at: i64,
    pub timezone: String,
    pub issuer: String,
    pub quantity: i64,
    pub sequence: i64,
    pub status: ScheduledDashboardLedgerStatus,
}

impl ScheduledDashboardLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDashboardLedgerStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDashboardLedgerStatus) -> bool {
        match self.status {
            ScheduledDashboardLedgerStatus::Idle => {
                match next {
                    ScheduledDashboardLedgerStatus::Warming | ScheduledDashboardLedgerStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDashboardLedgerStatus::Warming => {
                return next == ScheduledDashboardLedgerStatus::Stopped;
            }
            ScheduledDashboardLedgerStatus::Ready => {
                match next {
                    ScheduledDashboardLedgerStatus::Draining | ScheduledDashboardLedgerStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDashboardLedgerStatus::Draining => {
                return next == ScheduledDashboardLedgerStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledDashboardLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled dashboard ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled dashboard ledger", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.sequence as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDashboardLedger {
        ScheduledDashboardLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            version: self.version,
            expires_at: self.expires_at,
            timezone: self.timezone.clone(),
            issuer: self.issuer.clone(),
            quantity: self.quantity,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a scheduled dashboard ledger in status idle.
pub fn new_scheduled_dashboard_ledger(code: &str, title: &str) -> ScheduledDashboardLedger {
    ScheduledDashboardLedger {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        version: 0,
        expires_at: 0,
        timezone: "".to_string(),
        issuer: "".to_string(),
        quantity: 0,
        sequence: 0,
        status: ScheduledDashboardLedgerStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_dashboard_ledger("code-703", "title-470");
        assert_eq!(item.status, ScheduledDashboardLedgerStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_scheduled_dashboard_ledger("code-703", "title-470");
        if item.move_to(ScheduledDashboardLedgerStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledDashboardLedgerStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_dashboard_ledger("code-703", "title-470");
        item.status = ScheduledDashboardLedgerStatus::Stopped;
        assert!(
            item.move_to(ScheduledDashboardLedgerStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
