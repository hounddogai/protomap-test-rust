use super::*;

use acme_kit::text;

/// Where a capped slo slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSloSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CappedSloSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for CappedSloSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped slo slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloSlot {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub currency: String,
    pub quota_units: i64,
    pub tenant_id: String,
    pub status: CappedSloSlotStatus,
}

impl CappedSloSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSloSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSloSlotStatus) -> bool {
        match self.status {
            CappedSloSlotStatus::Draft => {
                match next {
                    CappedSloSlotStatus::Pending | CappedSloSlotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSloSlotStatus::Pending => {
                match next {
                    CappedSloSlotStatus::Active | CappedSloSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSloSlotStatus::Active => {
                return next == CappedSloSlotStatus::Suspended || next == CappedSloSlotStatus::Closed;
            }
            CappedSloSlotStatus::Suspended => {
                return next == CappedSloSlotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedSloSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped slo slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quota_units as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSloSlot {
        CappedSloSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            currency: self.currency.clone(),
            quota_units: self.quota_units,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped slo slot in status draft.
pub fn new_capped_slo_slot(code: &str, title: &str) -> CappedSloSlot {
    CappedSloSlot {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        currency: "".to_string(),
        quota_units: 0,
        tenant_id: "".to_string(),
        status: CappedSloSlotStatus::Draft,
    }
}
