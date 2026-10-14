use super::*;

use acme_kit::text;

/// Where a strict region pin adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictRegionPinAdjustmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictRegionPinAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictRegionPinAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict region pin adjustment as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictRegionPinAdjustment {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub fingerprint: String,
    pub segment: String,
    pub approved: bool,
    pub checksum: String,
    pub status: StrictRegionPinAdjustmentStatus,
}

impl StrictRegionPinAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictRegionPinAdjustmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictRegionPinAdjustmentStatus) -> bool {
        match self.status {
            StrictRegionPinAdjustmentStatus::Created => {
                return next == StrictRegionPinAdjustmentStatus::Validated
                    || next == StrictRegionPinAdjustmentStatus::Retired;
            }
            StrictRegionPinAdjustmentStatus::Validated => {
                match next {
                    StrictRegionPinAdjustmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictRegionPinAdjustmentStatus::Published => {
                match next {
                    StrictRegionPinAdjustmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictRegionPinAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict region pin adjustment {} cannot move from {} to {}",
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
        self.maximum
    }

    pub fn copy(&self) -> StrictRegionPinAdjustment {
        StrictRegionPinAdjustment {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            fingerprint: self.fingerprint.clone(),
            segment: self.segment.clone(),
            approved: self.approved,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a strict region pin adjustment in status created.
pub fn new_strict_region_pin_adjustment(code: &str, label: &str) -> StrictRegionPinAdjustment {
    StrictRegionPinAdjustment {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        fingerprint: "".to_string(),
        segment: "".to_string(),
        approved: false,
        checksum: "".to_string(),
        status: StrictRegionPinAdjustmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_region_pin_adjustment("code-713", "label-593");
        assert_eq!(item.status, StrictRegionPinAdjustmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_strict_region_pin_adjustment("code-713", "label-593");
        if item.move_to(StrictRegionPinAdjustmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictRegionPinAdjustmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_region_pin_adjustment("code-713", "label-593");
        item.status = StrictRegionPinAdjustmentStatus::Retired;
        assert!(
            item.move_to(StrictRegionPinAdjustmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
