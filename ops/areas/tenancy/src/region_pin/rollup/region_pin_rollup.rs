use super::*;

use acme_kit::text;

/// Where a region pin rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinRollupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionPinRollupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionPinRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin rollup as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinRollup {
    pub code: String,
    pub label: String,
    pub country: String,
    pub notes: String,
    pub name: String,
    pub quantity: i64,
    pub status: RegionPinRollupStatus,
}

impl RegionPinRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinRollupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinRollupStatus) -> bool {
        match self.status {
            RegionPinRollupStatus::Proposed => {
                return next == RegionPinRollupStatus::Approved || next == RegionPinRollupStatus::Rejected;
            }
            RegionPinRollupStatus::Approved => {
                return next == RegionPinRollupStatus::Rejected || next == RegionPinRollupStatus::Withdrawn;
            }
            RegionPinRollupStatus::Rejected => {
                match next {
                    RegionPinRollupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionPinRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> RegionPinRollup {
        RegionPinRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            notes: self.notes.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a region pin rollup in status proposed.
pub fn new_region_pin_rollup(code: &str, label: &str) -> RegionPinRollup {
    RegionPinRollup {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        notes: "".to_string(),
        name: "".to_string(),
        quantity: 0,
        status: RegionPinRollupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_region_pin_rollup("code-17", "label-304");
        assert_eq!(item.status, RegionPinRollupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_region_pin_rollup("code-17", "label-304");
        if item.move_to(RegionPinRollupStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinRollupStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_region_pin_rollup("code-17", "label-304");
        item.status = RegionPinRollupStatus::Withdrawn;
        assert!(
            item.move_to(RegionPinRollupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
