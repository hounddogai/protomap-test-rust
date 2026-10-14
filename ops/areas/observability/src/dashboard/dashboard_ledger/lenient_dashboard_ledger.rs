use super::*;

use acme_kit::text;

/// Where a lenient dashboard ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDashboardLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LenientDashboardLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for LenientDashboardLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient dashboard ledger as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDashboardLedger {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub discount: f64,
    pub expires_at: i64,
    pub weight: f64,
    pub ratio: f64,
    pub priority: i64,
    pub batch_size: i64,
    pub status: LenientDashboardLedgerStatus,
}

impl LenientDashboardLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDashboardLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDashboardLedgerStatus) -> bool {
        match self.status {
            LenientDashboardLedgerStatus::New => {
                match next {
                    LenientDashboardLedgerStatus::Running | LenientDashboardLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDashboardLedgerStatus::Queued => {
                return next == LenientDashboardLedgerStatus::Failed;
            }
            LenientDashboardLedgerStatus::Running => {
                return next == LenientDashboardLedgerStatus::Failed;
            }
            LenientDashboardLedgerStatus::Done => {
                return next == LenientDashboardLedgerStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientDashboardLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient dashboard ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.batch_size as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientDashboardLedger {
        LenientDashboardLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            discount: self.discount,
            expires_at: self.expires_at,
            weight: self.weight,
            ratio: self.ratio,
            priority: self.priority,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient dashboard ledger in status new.
pub fn new_lenient_dashboard_ledger(code: &str, label: &str) -> LenientDashboardLedger {
    LenientDashboardLedger {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        discount: 0.0,
        expires_at: 0,
        weight: 0.0,
        ratio: 0.0,
        priority: 0,
        batch_size: 0,
        status: LenientDashboardLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_lenient_dashboard_ledger("code-12", "label-532");
        assert_eq!(item.status, LenientDashboardLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_lenient_dashboard_ledger("code-12", "label-532");
        if item.move_to(LenientDashboardLedgerStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDashboardLedgerStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_lenient_dashboard_ledger("code-12", "label-532");
        item.status = LenientDashboardLedgerStatus::Failed;
        assert!(
            item.move_to(LenientDashboardLedgerStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
