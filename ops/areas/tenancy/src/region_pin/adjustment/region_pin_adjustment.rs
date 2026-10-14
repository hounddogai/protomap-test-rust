use super::*;

use acme_kit::text;

/// Where a region pin adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionPinAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionPinAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinAdjustment {
    pub code: String,
    pub title: String,
    pub label: String,
    pub ceiling: f64,
    pub checksum: String,
    pub attempts: i64,
    pub window_days: i64,
    pub namespace: String,
    pub tenant_id: String,
    pub status: RegionPinAdjustmentStatus,
}

impl RegionPinAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinAdjustmentStatus) -> bool {
        match self.status {
            RegionPinAdjustmentStatus::Proposed => {
                match next {
                    RegionPinAdjustmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinAdjustmentStatus::Approved => {
                return next == RegionPinAdjustmentStatus::Rejected || next == RegionPinAdjustmentStatus::Withdrawn;
            }
            RegionPinAdjustmentStatus::Rejected => {
                match next {
                    RegionPinAdjustmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionPinAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "region pin adjustment", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.window_days as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinAdjustment {
        RegionPinAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            checksum: self.checksum.clone(),
            attempts: self.attempts,
            window_days: self.window_days,
            namespace: self.namespace.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a region pin adjustment in status proposed.
pub fn new_region_pin_adjustment(code: &str, title: &str) -> RegionPinAdjustment {
    RegionPinAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        ceiling: 0.0,
        checksum: "".to_string(),
        attempts: 0,
        window_days: 0,
        namespace: "".to_string(),
        tenant_id: "".to_string(),
        status: RegionPinAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_region_pin_adjustment("code-849", "title-176");
        assert_eq!(item.status, RegionPinAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_region_pin_adjustment("code-849", "title-176");
        if item.move_to(RegionPinAdjustmentStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinAdjustmentStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_region_pin_adjustment("code-849", "title-176");
        item.status = RegionPinAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(RegionPinAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
