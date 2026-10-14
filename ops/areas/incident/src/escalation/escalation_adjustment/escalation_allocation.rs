use super::*;

use acme_kit::text;

/// Where a escalation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EscalationAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EscalationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation allocation as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAllocation {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub maximum: f64,
    pub owner_id: String,
    pub status: EscalationAllocationStatus,
}

impl EscalationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationAllocationStatus) -> bool {
        match self.status {
            EscalationAllocationStatus::Proposed => {
                match next {
                    EscalationAllocationStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationAllocationStatus::Approved => {
                match next {
                    EscalationAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationAllocationStatus::Rejected => {
                match next {
                    EscalationAllocationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: EscalationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "escalation allocation", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.expires_at as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationAllocation {
        EscalationAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            maximum: self.maximum,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a escalation allocation in status proposed.
pub fn new_escalation_allocation(code: &str, name: &str) -> EscalationAllocation {
    EscalationAllocation {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        maximum: 0.0,
        owner_id: "".to_string(),
        status: EscalationAllocationStatus::Proposed,
    }
}
