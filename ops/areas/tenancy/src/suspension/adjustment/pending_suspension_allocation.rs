use super::*;

use acme_kit::text;

/// Where a pending suspension allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSuspensionAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingSuspensionAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingSuspensionAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending suspension allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSuspensionAllocation {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub tenant_id: String,
    pub namespace: String,
    pub quota_units: i64,
    pub principal: String,
    pub starts_at: i64,
    pub discount: f64,
    pub weight: f64,
    pub contact_email: String,
    pub status: PendingSuspensionAllocationStatus,
}

impl PendingSuspensionAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSuspensionAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSuspensionAllocationStatus) -> bool {
        match self.status {
            PendingSuspensionAllocationStatus::Proposed => {
                match next {
                    PendingSuspensionAllocationStatus::Rejected | PendingSuspensionAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSuspensionAllocationStatus::Approved => {
                match next {
                    PendingSuspensionAllocationStatus::Rejected | PendingSuspensionAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSuspensionAllocationStatus::Rejected => {
                match next {
                    PendingSuspensionAllocationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingSuspensionAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending suspension allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.starts_at as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSuspensionAllocation {
        PendingSuspensionAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            tenant_id: self.tenant_id.clone(),
            namespace: self.namespace.clone(),
            quota_units: self.quota_units,
            principal: self.principal.clone(),
            starts_at: self.starts_at,
            discount: self.discount,
            weight: self.weight,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a pending suspension allocation in status proposed.
pub fn new_pending_suspension_allocation(code: &str, label: &str) -> PendingSuspensionAllocation {
    PendingSuspensionAllocation {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        tenant_id: "".to_string(),
        namespace: "".to_string(),
        quota_units: 0,
        principal: "".to_string(),
        starts_at: 0,
        discount: 0.0,
        weight: 0.0,
        contact_email: "".to_string(),
        status: PendingSuspensionAllocationStatus::Proposed,
    }
}
