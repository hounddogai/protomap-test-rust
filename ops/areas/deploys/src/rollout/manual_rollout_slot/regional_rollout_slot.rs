use super::*;

use acme_kit::text;

/// Where a regional rollout slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalRolloutSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RegionalRolloutSlotStatus {
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

impl std::fmt::Display for RegionalRolloutSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional rollout slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRolloutSlot {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub quantity: i64,
    pub quota_units: i64,
    pub notes: String,
    pub discount: f64,
    pub status: RegionalRolloutSlotStatus,
}

impl RegionalRolloutSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalRolloutSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalRolloutSlotStatus) -> bool {
        match self.status {
            RegionalRolloutSlotStatus::Draft => {
                return next == RegionalRolloutSlotStatus::Active;
            }
            RegionalRolloutSlotStatus::Pending => {
                match next {
                    RegionalRolloutSlotStatus::Active | RegionalRolloutSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRolloutSlotStatus::Active => {
                match next {
                    RegionalRolloutSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalRolloutSlotStatus::Suspended => {
                return next == RegionalRolloutSlotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalRolloutSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional rollout slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.discount + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalRolloutSlot {
        RegionalRolloutSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            quantity: self.quantity,
            quota_units: self.quota_units,
            notes: self.notes.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional rollout slot in status draft.
pub fn new_regional_rollout_slot(code: &str, label: &str) -> RegionalRolloutSlot {
    RegionalRolloutSlot {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        quantity: 0,
        quota_units: 0,
        notes: "".to_string(),
        discount: 0.0,
        status: RegionalRolloutSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_regional_rollout_slot("code-958", "label-419");
        assert_eq!(item.status, RegionalRolloutSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_regional_rollout_slot("code-958", "label-419");
        if item.move_to(RegionalRolloutSlotStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalRolloutSlotStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_regional_rollout_slot("code-958", "label-419");
        item.status = RegionalRolloutSlotStatus::Closed;
        assert!(
            item.move_to(RegionalRolloutSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
