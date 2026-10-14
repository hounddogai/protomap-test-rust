use super::*;

use acme_kit::text;

/// Where a policy ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PolicyLedgerStatus {
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

impl std::fmt::Display for PolicyLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy ledger as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyLedger {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub region: String,
    pub weight: f64,
    pub budget: f64,
    pub quota_units: i64,
    pub ttl_seconds: i64,
    pub batch_size: i64,
    pub segment: String,
    pub summary: String,
    pub status: PolicyLedgerStatus,
}

impl PolicyLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyLedgerStatus) -> bool {
        match self.status {
            PolicyLedgerStatus::New => {
                return next == PolicyLedgerStatus::Queued;
            }
            PolicyLedgerStatus::Queued => {
                match next {
                    PolicyLedgerStatus::Running | PolicyLedgerStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyLedgerStatus::Running => {
                return next == PolicyLedgerStatus::Done;
            }
            PolicyLedgerStatus::Done => {
                return next == PolicyLedgerStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "policy ledger", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.quota_units as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicyLedger {
        PolicyLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            region: self.region.clone(),
            weight: self.weight,
            budget: self.budget,
            quota_units: self.quota_units,
            ttl_seconds: self.ttl_seconds,
            batch_size: self.batch_size,
            segment: self.segment.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a policy ledger in status new.
pub fn new_policy_ledger(code: &str, label: &str) -> PolicyLedger {
    PolicyLedger {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        region: "".to_string(),
        weight: 0.0,
        budget: 0.0,
        quota_units: 0,
        ttl_seconds: 0,
        batch_size: 0,
        segment: "".to_string(),
        summary: "".to_string(),
        status: PolicyLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_policy_ledger("code-708", "label-824");
        assert_eq!(item.status, PolicyLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_policy_ledger("code-708", "label-824");
        if item.move_to(PolicyLedgerStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyLedgerStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_policy_ledger("code-708", "label-824");
        item.status = PolicyLedgerStatus::Failed;
        assert!(item.move_to(PolicyLedgerStatus::New).is_err(), "expected move to fail");
    }
}
