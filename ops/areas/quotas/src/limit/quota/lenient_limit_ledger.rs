use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient limit ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientLimitLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientLimitLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LenientLimitLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient limit ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientLimitLedger {
    pub code: String,
    pub name: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub duration_minutes: i64,
    pub owner_id: String,
    pub ratio: f64,
    pub status: LenientLimitLedgerStatus,
}

impl LenientLimitLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientLimitLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientLimitLedgerStatus) -> bool {
        match self.status {
            LenientLimitLedgerStatus::Planned => {
                return next == LenientLimitLedgerStatus::Scheduled;
            }
            LenientLimitLedgerStatus::Scheduled => {
                match next {
                    LenientLimitLedgerStatus::InProgress | LenientLimitLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientLimitLedgerStatus::InProgress => {
                match next {
                    LenientLimitLedgerStatus::Completed | LenientLimitLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientLimitLedgerStatus::Completed => {
                return next == LenientLimitLedgerStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientLimitLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient limit ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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

    pub fn net_value(&self) -> f64 {
        ((self.duration_minutes as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientLimitLedger {
        LenientLimitLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            duration_minutes: self.duration_minutes,
            owner_id: self.owner_id.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient limit ledger in status planned.
pub fn new_lenient_limit_ledger(code: &str, name: &str) -> LenientLimitLedger {
    LenientLimitLedger {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        attributes: BTreeMap::new(),
        duration_minutes: 0,
        owner_id: "".to_string(),
        ratio: 0.0,
        status: LenientLimitLedgerStatus::Planned,
    }
}
