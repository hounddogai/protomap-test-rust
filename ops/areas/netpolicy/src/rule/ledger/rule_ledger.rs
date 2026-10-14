use super::*;

use acme_kit::text;

/// Where a rule ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RuleLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RuleLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule ledger as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleLedger {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub discount: f64,
    pub source: String,
    pub tags: Vec<String>,
    pub checksum: String,
    pub threshold: f64,
    pub visible: bool,
    pub port: i64,
    pub status: RuleLedgerStatus,
}

impl RuleLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleLedgerStatus) -> bool {
        match self.status {
            RuleLedgerStatus::Requested => {
                return next == RuleLedgerStatus::Granted;
            }
            RuleLedgerStatus::Granted => {
                return next == RuleLedgerStatus::Revoked || next == RuleLedgerStatus::Expired;
            }
            RuleLedgerStatus::Revoked => {
                return next == RuleLedgerStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_total(&self) -> f64 {
        ((self.threshold - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleLedger {
        RuleLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            discount: self.discount,
            source: self.source.clone(),
            tags: self.tags.clone(),
            checksum: self.checksum.clone(),
            threshold: self.threshold,
            visible: self.visible,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rule ledger in status requested.
pub fn new_rule_ledger(code: &str, title: &str) -> RuleLedger {
    RuleLedger {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        discount: 0.0,
        source: "".to_string(),
        tags: Vec::new(),
        checksum: "".to_string(),
        threshold: 0.0,
        visible: false,
        port: 0,
        status: RuleLedgerStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rule_ledger("code-549", "title-328");
        assert_eq!(item.status, RuleLedgerStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rule_ledger("code-549", "title-328");
        if item.move_to(RuleLedgerStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleLedgerStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rule_ledger("code-549", "title-328");
        item.status = RuleLedgerStatus::Expired;
        assert!(
            item.move_to(RuleLedgerStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
