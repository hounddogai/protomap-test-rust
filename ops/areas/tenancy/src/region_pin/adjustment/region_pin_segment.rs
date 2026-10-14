use super::*;

use acme_kit::text;

/// Where a region pin segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionPinSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionPinSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSegment {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub scope: String,
    pub summary: String,
    pub rotation_days: i64,
    pub country: String,
    pub quantity: i64,
    pub ceiling: f64,
    pub status: RegionPinSegmentStatus,
}

impl RegionPinSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinSegmentStatus) -> bool {
        match self.status {
            RegionPinSegmentStatus::Proposed => {
                match next {
                    RegionPinSegmentStatus::Approved | RegionPinSegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinSegmentStatus::Approved => {
                match next {
                    RegionPinSegmentStatus::Rejected | RegionPinSegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinSegmentStatus::Rejected => {
                return next == RegionPinSegmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.ceiling * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinSegment {
        RegionPinSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            scope: self.scope.clone(),
            summary: self.summary.clone(),
            rotation_days: self.rotation_days,
            country: self.country.clone(),
            quantity: self.quantity,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a region pin segment in status proposed.
pub fn new_region_pin_segment(code: &str, label: &str) -> RegionPinSegment {
    RegionPinSegment {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        scope: "".to_string(),
        summary: "".to_string(),
        rotation_days: 0,
        country: "".to_string(),
        quantity: 0,
        ceiling: 0.0,
        status: RegionPinSegmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_region_pin_segment("code-383", "label-971");
        assert_eq!(item.status, RegionPinSegmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_region_pin_segment("code-383", "label-971");
        if item.move_to(RegionPinSegmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinSegmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_region_pin_segment("code-383", "label-971");
        item.status = RegionPinSegmentStatus::Withdrawn;
        assert!(
            item.move_to(RegionPinSegmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
