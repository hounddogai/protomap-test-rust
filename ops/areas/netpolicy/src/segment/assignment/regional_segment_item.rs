use super::*;

use acme_kit::text;

/// Where a regional segment item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSegmentItemStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalSegmentItemStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalSegmentItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional segment item. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSegmentItem {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub summary: String,
    pub description: String,
    pub rate: f64,
    pub active: bool,
    pub region: String,
    pub timezone: String,
    pub status: RegionalSegmentItemStatus,
}

impl RegionalSegmentItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSegmentItemStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSegmentItemStatus) -> bool {
        match self.status {
            RegionalSegmentItemStatus::Proposed => {
                return next == RegionalSegmentItemStatus::Approved || next == RegionalSegmentItemStatus::Rejected;
            }
            RegionalSegmentItemStatus::Approved => {
                match next {
                    RegionalSegmentItemStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalSegmentItemStatus::Rejected => {
                return next == RegionalSegmentItemStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalSegmentItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional segment item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        (self.rate * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSegmentItem {
        RegionalSegmentItem {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            summary: self.summary.clone(),
            description: self.description.clone(),
            rate: self.rate,
            active: self.active,
            region: self.region.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional segment item in status proposed.
pub fn new_regional_segment_item(code: &str, label: &str) -> RegionalSegmentItem {
    RegionalSegmentItem {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        summary: "".to_string(),
        description: "".to_string(),
        rate: 0.0,
        active: false,
        region: "".to_string(),
        timezone: "".to_string(),
        status: RegionalSegmentItemStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_segment_item("code-892", "label-357");
        assert_eq!(item.status, RegionalSegmentItemStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_regional_segment_item("code-892", "label-357");
        if item.move_to(RegionalSegmentItemStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalSegmentItemStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_segment_item("code-892", "label-357");
        item.status = RegionalSegmentItemStatus::Withdrawn;
        assert!(
            item.move_to(RegionalSegmentItemStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
