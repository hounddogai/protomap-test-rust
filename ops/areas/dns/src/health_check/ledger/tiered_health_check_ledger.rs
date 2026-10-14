use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a tiered health check ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredHealthCheckLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredHealthCheckLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredHealthCheckLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered health check ledger as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckLedger {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub revision: i64,
    pub ceiling: f64,
    pub scope: String,
    pub discount: f64,
    pub country: String,
    pub cost: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: TieredHealthCheckLedgerStatus,
}

impl TieredHealthCheckLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredHealthCheckLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredHealthCheckLedgerStatus) -> bool {
        match self.status {
            TieredHealthCheckLedgerStatus::Created => {
                return next == TieredHealthCheckLedgerStatus::Published;
            }
            TieredHealthCheckLedgerStatus::Validated => {
                return next == TieredHealthCheckLedgerStatus::Published
                    || next == TieredHealthCheckLedgerStatus::Retired;
            }
            TieredHealthCheckLedgerStatus::Published => {
                return next == TieredHealthCheckLedgerStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredHealthCheckLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered health check ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered health check ledger", self.code)
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

    pub fn adjusted_value(&self) -> f64 {
        ((self.ceiling - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredHealthCheckLedger {
        TieredHealthCheckLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            revision: self.revision,
            ceiling: self.ceiling,
            scope: self.scope.clone(),
            discount: self.discount,
            country: self.country.clone(),
            cost: self.cost,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered health check ledger in status created.
pub fn new_tiered_health_check_ledger(code: &str, label: &str) -> TieredHealthCheckLedger {
    TieredHealthCheckLedger {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        revision: 0,
        ceiling: 0.0,
        scope: "".to_string(),
        discount: 0.0,
        country: "".to_string(),
        cost: 0.0,
        attributes: BTreeMap::new(),
        status: TieredHealthCheckLedgerStatus::Created,
    }
}
