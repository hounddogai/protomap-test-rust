use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a grant ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GrantLedgerStatus {
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

impl std::fmt::Display for GrantLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantLedger {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub attributes: BTreeMap<String, String>,
    pub channel: String,
    pub quota_units: i64,
    pub segment: String,
    pub currency: String,
    pub status: GrantLedgerStatus,
}

impl GrantLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantLedgerStatus) -> bool {
        match self.status {
            GrantLedgerStatus::New => {
                return next == GrantLedgerStatus::Done;
            }
            GrantLedgerStatus::Queued => {
                match next {
                    GrantLedgerStatus::Running | GrantLedgerStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantLedgerStatus::Running => {
                return next == GrantLedgerStatus::Failed;
            }
            GrantLedgerStatus::Done => {
                match next {
                    GrantLedgerStatus::Failed => {
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

    pub fn move_to(&mut self, next: GrantLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> GrantLedger {
        GrantLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            attributes: self.attributes.clone(),
            channel: self.channel.clone(),
            quota_units: self.quota_units,
            segment: self.segment.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a grant ledger in status new.
pub fn new_grant_ledger(code: &str, label: &str) -> GrantLedger {
    GrantLedger {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        attributes: BTreeMap::new(),
        channel: "".to_string(),
        quota_units: 0,
        segment: "".to_string(),
        currency: "".to_string(),
        status: GrantLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_grant_ledger("code-137", "label-586");
        assert_eq!(item.status, GrantLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_grant_ledger("code-137", "label-586");
        if item.move_to(GrantLedgerStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantLedgerStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_grant_ledger("code-137", "label-586");
        item.status = GrantLedgerStatus::Failed;
        assert!(item.move_to(GrantLedgerStatus::New).is_err(), "expected move to fail");
    }
}
