use super::*;

use acme_kit::text;

/// Where a legacy identity ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyIdentityLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LegacyIdentityLedgerStatus {
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

impl std::fmt::Display for LegacyIdentityLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy identity ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIdentityLedger {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub external_ref: String,
    pub channel: String,
    pub namespace: String,
    pub status: LegacyIdentityLedgerStatus,
}

impl LegacyIdentityLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyIdentityLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyIdentityLedgerStatus) -> bool {
        match self.status {
            LegacyIdentityLedgerStatus::Planned => {
                match next {
                    LegacyIdentityLedgerStatus::InProgress | LegacyIdentityLedgerStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyIdentityLedgerStatus::Scheduled => {
                return next == LegacyIdentityLedgerStatus::Completed;
            }
            LegacyIdentityLedgerStatus::InProgress => {
                match next {
                    LegacyIdentityLedgerStatus::Completed | LegacyIdentityLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyIdentityLedgerStatus::Completed => {
                match next {
                    LegacyIdentityLedgerStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LegacyIdentityLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy identity ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> LegacyIdentityLedger {
        LegacyIdentityLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            external_ref: self.external_ref.clone(),
            channel: self.channel.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a legacy identity ledger in status planned.
pub fn new_legacy_identity_ledger(code: &str, label: &str) -> LegacyIdentityLedger {
    LegacyIdentityLedger {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        external_ref: "".to_string(),
        channel: "".to_string(),
        namespace: "".to_string(),
        status: LegacyIdentityLedgerStatus::Planned,
    }
}
