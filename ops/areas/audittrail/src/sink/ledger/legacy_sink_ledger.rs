use super::*;

use acme_kit::text;

/// Where a legacy sink ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacySinkLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacySinkLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacySinkLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy sink ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySinkLedger {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub cluster: String,
    pub source: String,
    pub quota_units: i64,
    pub status: LegacySinkLedgerStatus,
}

impl LegacySinkLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacySinkLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacySinkLedgerStatus) -> bool {
        match self.status {
            LegacySinkLedgerStatus::Created => {
                match next {
                    LegacySinkLedgerStatus::Validated | LegacySinkLedgerStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySinkLedgerStatus::Validated => {
                match next {
                    LegacySinkLedgerStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySinkLedgerStatus::Published => {
                match next {
                    LegacySinkLedgerStatus::Retired => {
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

    pub fn move_to(&mut self, next: LegacySinkLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy sink ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> LegacySinkLedger {
        LegacySinkLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            cluster: self.cluster.clone(),
            source: self.source.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy sink ledger in status created.
pub fn new_legacy_sink_ledger(code: &str, name: &str) -> LegacySinkLedger {
    LegacySinkLedger {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        cluster: "".to_string(),
        source: "".to_string(),
        quota_units: 0,
        status: LegacySinkLedgerStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_legacy_sink_ledger("code-92", "name-763");
        assert_eq!(item.status, LegacySinkLedgerStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_legacy_sink_ledger("code-92", "name-763");
        if item.move_to(LegacySinkLedgerStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacySinkLedgerStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_legacy_sink_ledger("code-92", "name-763");
        item.status = LegacySinkLedgerStatus::Retired;
        assert!(
            item.move_to(LegacySinkLedgerStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
