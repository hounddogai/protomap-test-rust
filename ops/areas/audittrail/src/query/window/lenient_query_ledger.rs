use super::*;

use acme_kit::text;

/// Where a lenient query ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientQueryLedgerStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientQueryLedgerStatus {
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

impl std::fmt::Display for LenientQueryLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient query ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientQueryLedger {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub ratio: f64,
    pub rate: f64,
    pub revision: i64,
    pub channel: String,
    pub segment: String,
    pub cluster: String,
    pub status: LenientQueryLedgerStatus,
}

impl LenientQueryLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientQueryLedgerStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientQueryLedgerStatus) -> bool {
        match self.status {
            LenientQueryLedgerStatus::Idle => {
                return next == LenientQueryLedgerStatus::Ready || next == LenientQueryLedgerStatus::Stopped;
            }
            LenientQueryLedgerStatus::Warming => {
                return next == LenientQueryLedgerStatus::Draining;
            }
            LenientQueryLedgerStatus::Ready => {
                return next == LenientQueryLedgerStatus::Draining || next == LenientQueryLedgerStatus::Stopped;
            }
            LenientQueryLedgerStatus::Draining => {
                return next == LenientQueryLedgerStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientQueryLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient query ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ratio - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientQueryLedger {
        LenientQueryLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            ratio: self.ratio,
            rate: self.rate,
            revision: self.revision,
            channel: self.channel.clone(),
            segment: self.segment.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a lenient query ledger in status idle.
pub fn new_lenient_query_ledger(code: &str, label: &str) -> LenientQueryLedger {
    LenientQueryLedger {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        ratio: 0.0,
        rate: 0.0,
        revision: 0,
        channel: "".to_string(),
        segment: "".to_string(),
        cluster: "".to_string(),
        status: LenientQueryLedgerStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_query_ledger("code-19", "label-767");
        assert_eq!(item.status, LenientQueryLedgerStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_lenient_query_ledger("code-19", "label-767");
        if item.move_to(LenientQueryLedgerStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientQueryLedgerStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_query_ledger("code-19", "label-767");
        item.status = LenientQueryLedgerStatus::Stopped;
        assert!(
            item.move_to(LenientQueryLedgerStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
