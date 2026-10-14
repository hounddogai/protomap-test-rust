use super::*;

use acme_kit::text;

/// Where a evidence allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EvidenceAllocationStatus {
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

impl std::fmt::Display for EvidenceAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAllocation {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub discount: f64,
    pub reviewer: String,
    pub status: EvidenceAllocationStatus,
}

impl EvidenceAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceAllocationStatus) -> bool {
        match self.status {
            EvidenceAllocationStatus::Planned => {
                match next {
                    EvidenceAllocationStatus::Completed | EvidenceAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceAllocationStatus::Scheduled => {
                match next {
                    EvidenceAllocationStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceAllocationStatus::InProgress => {
                match next {
                    EvidenceAllocationStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceAllocationStatus::Completed => {
                match next {
                    EvidenceAllocationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EvidenceAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "evidence allocation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> EvidenceAllocation {
        EvidenceAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            discount: self.discount,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a evidence allocation in status planned.
pub fn new_evidence_allocation(code: &str, label: &str) -> EvidenceAllocation {
    EvidenceAllocation {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        discount: 0.0,
        reviewer: "".to_string(),
        status: EvidenceAllocationStatus::Planned,
    }
}
