use super::*;

use acme_kit::text;

/// Where a rolling silence ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingSilenceLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingSilenceLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingSilenceLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling silence ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceLedger {
    pub code: String,
    pub title: String,
    pub label: String,
    pub external_ref: String,
    pub cluster: String,
    pub ceiling: f64,
    pub locale: String,
    pub key_id: String,
    pub namespace: String,
    pub region: String,
    pub discount: f64,
    pub status: RollingSilenceLedgerStatus,
}

impl RollingSilenceLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingSilenceLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingSilenceLedgerStatus) -> bool {
        match self.status {
            RollingSilenceLedgerStatus::Requested => {
                return next == RollingSilenceLedgerStatus::Granted;
            }
            RollingSilenceLedgerStatus::Granted => {
                match next {
                    RollingSilenceLedgerStatus::Revoked | RollingSilenceLedgerStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingSilenceLedgerStatus::Revoked => {
                return next == RollingSilenceLedgerStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingSilenceLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling silence ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.discount * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingSilenceLedger {
        RollingSilenceLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            cluster: self.cluster.clone(),
            ceiling: self.ceiling,
            locale: self.locale.clone(),
            key_id: self.key_id.clone(),
            namespace: self.namespace.clone(),
            region: self.region.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling silence ledger in status requested.
pub fn new_rolling_silence_ledger(code: &str, title: &str) -> RollingSilenceLedger {
    RollingSilenceLedger {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        external_ref: "".to_string(),
        cluster: "".to_string(),
        ceiling: 0.0,
        locale: "".to_string(),
        key_id: "".to_string(),
        namespace: "".to_string(),
        region: "".to_string(),
        discount: 0.0,
        status: RollingSilenceLedgerStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_silence_ledger("code-262", "title-677");
        assert_eq!(item.status, RollingSilenceLedgerStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rolling_silence_ledger("code-262", "title-677");
        if item.move_to(RollingSilenceLedgerStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingSilenceLedgerStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_silence_ledger("code-262", "title-677");
        item.status = RollingSilenceLedgerStatus::Expired;
        assert!(
            item.move_to(RollingSilenceLedgerStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
