use super::*;

use acme_kit::text;

/// Where a zone allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneAllocationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ZoneAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ZoneAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAllocation {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub created_at: i64,
    pub score: f64,
    pub category: String,
    pub amount: f64,
    pub status: ZoneAllocationStatus,
}

impl ZoneAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneAllocationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneAllocationStatus) -> bool {
        match self.status {
            ZoneAllocationStatus::Proposed => {
                match next {
                    ZoneAllocationStatus::Approved | ZoneAllocationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneAllocationStatus::Approved => {
                match next {
                    ZoneAllocationStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneAllocationStatus::Rejected => {
                return next == ZoneAllocationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.created_at as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneAllocation {
        ZoneAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            created_at: self.created_at,
            score: self.score,
            category: self.category.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a zone allocation in status proposed.
pub fn new_zone_allocation(code: &str, label: &str) -> ZoneAllocation {
    ZoneAllocation {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        created_at: 0,
        score: 0.0,
        category: "".to_string(),
        amount: 0.0,
        status: ZoneAllocationStatus::Proposed,
    }
}
