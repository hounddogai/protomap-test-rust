use super::*;

use acme_kit::text;

/// Where a legacy policy ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPolicyLedgerStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyPolicyLedgerStatus {
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

impl std::fmt::Display for LegacyPolicyLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy policy ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyLedger {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub priority: i64,
    pub fingerprint: String,
    pub status: LegacyPolicyLedgerStatus,
}

impl LegacyPolicyLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPolicyLedgerStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPolicyLedgerStatus) -> bool {
        match self.status {
            LegacyPolicyLedgerStatus::Idle => {
                match next {
                    LegacyPolicyLedgerStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPolicyLedgerStatus::Warming => {
                match next {
                    LegacyPolicyLedgerStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPolicyLedgerStatus::Ready => {
                match next {
                    LegacyPolicyLedgerStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPolicyLedgerStatus::Draining => {
                match next {
                    LegacyPolicyLedgerStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LegacyPolicyLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy policy ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy policy ledger", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> LegacyPolicyLedger {
        LegacyPolicyLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            priority: self.priority,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy policy ledger in status idle.
pub fn new_legacy_policy_ledger(code: &str, label: &str) -> LegacyPolicyLedger {
    LegacyPolicyLedger {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        priority: 0,
        fingerprint: "".to_string(),
        status: LegacyPolicyLedgerStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_policy_ledger("code-10", "label-872");
        assert_eq!(item.status, LegacyPolicyLedgerStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_legacy_policy_ledger("code-10", "label-872");
        if item.move_to(LegacyPolicyLedgerStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyPolicyLedgerStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_policy_ledger("code-10", "label-872");
        item.status = LegacyPolicyLedgerStatus::Stopped;
        assert!(
            item.move_to(LegacyPolicyLedgerStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
