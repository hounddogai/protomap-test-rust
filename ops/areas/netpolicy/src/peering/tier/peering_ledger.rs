use super::*;

use acme_kit::text;

/// Where a peering ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PeeringLedgerStatus {
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

impl std::fmt::Display for PeeringLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringLedger {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub namespace: String,
    pub region: String,
    pub status: PeeringLedgerStatus,
}

impl PeeringLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringLedgerStatus) -> bool {
        match self.status {
            PeeringLedgerStatus::New => {
                return next == PeeringLedgerStatus::Queued || next == PeeringLedgerStatus::Done;
            }
            PeeringLedgerStatus::Queued => {
                match next {
                    PeeringLedgerStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringLedgerStatus::Running => {
                match next {
                    PeeringLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringLedgerStatus::Done => {
                return next == PeeringLedgerStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> PeeringLedger {
        PeeringLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            namespace: self.namespace.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a peering ledger in status new.
pub fn new_peering_ledger(code: &str, label: &str) -> PeeringLedger {
    PeeringLedger {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        namespace: "".to_string(),
        region: "".to_string(),
        status: PeeringLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_peering_ledger("code-859", "label-2");
        assert_eq!(item.status, PeeringLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_peering_ledger("code-859", "label-2");
        if item.move_to(PeeringLedgerStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringLedgerStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_peering_ledger("code-859", "label-2");
        item.status = PeeringLedgerStatus::Failed;
        assert!(item.move_to(PeeringLedgerStatus::New).is_err(), "expected move to fail");
    }
}
