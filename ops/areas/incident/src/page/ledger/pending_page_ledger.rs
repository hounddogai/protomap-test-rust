use super::*;

use acme_kit::text;

/// Where a pending page ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPageLedgerStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingPageLedgerStatus {
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

impl std::fmt::Display for PendingPageLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending page ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageLedger {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub weight: f64,
    pub cluster: String,
    pub tags: Vec<String>,
    pub attempts: i64,
    pub unit: String,
    pub floor_value: f64,
    pub margin: f64,
    pub status: PendingPageLedgerStatus,
}

impl PendingPageLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPageLedgerStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPageLedgerStatus) -> bool {
        match self.status {
            PendingPageLedgerStatus::Idle => {
                match next {
                    PendingPageLedgerStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPageLedgerStatus::Warming => {
                match next {
                    PendingPageLedgerStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingPageLedgerStatus::Ready => {
                return next == PendingPageLedgerStatus::Draining;
            }
            PendingPageLedgerStatus::Draining => {
                match next {
                    PendingPageLedgerStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingPageLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending page ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending page ledger", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        ((self.quantity as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPageLedger {
        PendingPageLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            weight: self.weight,
            cluster: self.cluster.clone(),
            tags: self.tags.clone(),
            attempts: self.attempts,
            unit: self.unit.clone(),
            floor_value: self.floor_value,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a pending page ledger in status idle.
pub fn new_pending_page_ledger(code: &str, name: &str) -> PendingPageLedger {
    PendingPageLedger {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        weight: 0.0,
        cluster: "".to_string(),
        tags: Vec::new(),
        attempts: 0,
        unit: "".to_string(),
        floor_value: 0.0,
        margin: 0.0,
        status: PendingPageLedgerStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_page_ledger("code-489", "name-699");
        assert_eq!(item.status, PendingPageLedgerStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_pending_page_ledger("code-489", "name-699");
        if item.move_to(PendingPageLedgerStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingPageLedgerStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_page_ledger("code-489", "name-699");
        item.status = PendingPageLedgerStatus::Stopped;
        assert!(
            item.move_to(PendingPageLedgerStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
