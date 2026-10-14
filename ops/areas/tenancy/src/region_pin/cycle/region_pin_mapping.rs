use super::*;

use acme_kit::text;

/// Where a region pin mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionPinMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionPinMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin mapping as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinMapping {
    pub code: String,
    pub label: String,
    pub category: String,
    pub weight: f64,
    pub locale: String,
    pub status: RegionPinMappingStatus,
}

impl RegionPinMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinMappingStatus) -> bool {
        match self.status {
            RegionPinMappingStatus::Proposed => {
                match next {
                    RegionPinMappingStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinMappingStatus::Approved => {
                match next {
                    RegionPinMappingStatus::Rejected | RegionPinMappingStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinMappingStatus::Rejected => {
                return next == RegionPinMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin mapping {} cannot move from {} to {}",
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
        self.weight
    }

    pub fn copy(&self) -> RegionPinMapping {
        RegionPinMapping {
            code: self.code.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            weight: self.weight,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a region pin mapping in status proposed.
pub fn new_region_pin_mapping(code: &str, label: &str) -> RegionPinMapping {
    RegionPinMapping {
        code: code.to_string(),
        label: label.to_string(),
        category: "".to_string(),
        weight: 0.0,
        locale: "".to_string(),
        status: RegionPinMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_region_pin_mapping("code-473", "label-800");
        assert_eq!(item.status, RegionPinMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_region_pin_mapping("code-473", "label-800");
        if item.move_to(RegionPinMappingStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinMappingStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_region_pin_mapping("code-473", "label-800");
        item.status = RegionPinMappingStatus::Withdrawn;
        assert!(
            item.move_to(RegionPinMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
