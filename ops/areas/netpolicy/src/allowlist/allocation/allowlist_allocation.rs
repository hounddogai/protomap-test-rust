use super::*;

use acme_kit::text;

/// Where a allowlist allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AllowlistAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AllowlistAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAllocation {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub retries: i64,
    pub host: String,
    pub expires_at: i64,
    pub source: String,
    pub revision: i64,
    pub discount: f64,
    pub status: AllowlistAllocationStatus,
}

impl AllowlistAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistAllocationStatus) -> bool {
        match self.status {
            AllowlistAllocationStatus::Proposed => {
                match next {
                    AllowlistAllocationStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistAllocationStatus::Approved => {
                match next {
                    AllowlistAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistAllocationStatus::Rejected => {
                return next == AllowlistAllocationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.revision as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistAllocation {
        AllowlistAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            retries: self.retries,
            host: self.host.clone(),
            expires_at: self.expires_at,
            source: self.source.clone(),
            revision: self.revision,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a allowlist allocation in status proposed.
pub fn new_allowlist_allocation(code: &str, label: &str) -> AllowlistAllocation {
    AllowlistAllocation {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        retries: 0,
        host: "".to_string(),
        expires_at: 0,
        source: "".to_string(),
        revision: 0,
        discount: 0.0,
        status: AllowlistAllocationStatus::Proposed,
    }
}
