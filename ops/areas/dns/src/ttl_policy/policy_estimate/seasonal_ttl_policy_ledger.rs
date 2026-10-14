use super::*;

use acme_kit::text;

/// Where a seasonal ttl policy ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTtlPolicyLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalTtlPolicyLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalTtlPolicyLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal ttl policy ledger as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyLedger {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub tenant_id: String,
    pub revision: i64,
    pub host: String,
    pub contact_email: String,
    pub status: SeasonalTtlPolicyLedgerStatus,
}

impl SeasonalTtlPolicyLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTtlPolicyLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTtlPolicyLedgerStatus) -> bool {
        match self.status {
            SeasonalTtlPolicyLedgerStatus::Proposed => {
                match next {
                    SeasonalTtlPolicyLedgerStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTtlPolicyLedgerStatus::Approved => {
                match next {
                    SeasonalTtlPolicyLedgerStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTtlPolicyLedgerStatus::Rejected => {
                return next == SeasonalTtlPolicyLedgerStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalTtlPolicyLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal ttl policy ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.discount + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTtlPolicyLedger {
        SeasonalTtlPolicyLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            tenant_id: self.tenant_id.clone(),
            revision: self.revision,
            host: self.host.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal ttl policy ledger in status proposed.
pub fn new_seasonal_ttl_policy_ledger(code: &str, label: &str) -> SeasonalTtlPolicyLedger {
    SeasonalTtlPolicyLedger {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        tenant_id: "".to_string(),
        revision: 0,
        host: "".to_string(),
        contact_email: "".to_string(),
        status: SeasonalTtlPolicyLedgerStatus::Proposed,
    }
}
