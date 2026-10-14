use super::*;

use acme_kit::text;

/// Where a regional verifier segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalVerifierSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalVerifierSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalVerifierSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional verifier segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierSegment {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub issuer: String,
    pub floor_value: f64,
    pub grace_days: i64,
    pub audience: String,
    pub contact_email: String,
    pub version: i64,
    pub unit: String,
    pub status: RegionalVerifierSegmentStatus,
}

impl RegionalVerifierSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalVerifierSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalVerifierSegmentStatus) -> bool {
        match self.status {
            RegionalVerifierSegmentStatus::Proposed => {
                match next {
                    RegionalVerifierSegmentStatus::Approved | RegionalVerifierSegmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalVerifierSegmentStatus::Approved => {
                return next == RegionalVerifierSegmentStatus::Rejected;
            }
            RegionalVerifierSegmentStatus::Rejected => {
                return next == RegionalVerifierSegmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalVerifierSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional verifier segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional verifier segment", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.quantity as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalVerifierSegment {
        RegionalVerifierSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            issuer: self.issuer.clone(),
            floor_value: self.floor_value,
            grace_days: self.grace_days,
            audience: self.audience.clone(),
            contact_email: self.contact_email.clone(),
            version: self.version,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a regional verifier segment in status proposed.
pub fn new_regional_verifier_segment(code: &str, label: &str) -> RegionalVerifierSegment {
    RegionalVerifierSegment {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        issuer: "".to_string(),
        floor_value: 0.0,
        grace_days: 0,
        audience: "".to_string(),
        contact_email: "".to_string(),
        version: 0,
        unit: "".to_string(),
        status: RegionalVerifierSegmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_verifier_segment("code-791", "label-233");
        assert_eq!(item.status, RegionalVerifierSegmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_regional_verifier_segment("code-791", "label-233");
        if item.move_to(RegionalVerifierSegmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalVerifierSegmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_verifier_segment("code-791", "label-233");
        item.status = RegionalVerifierSegmentStatus::Withdrawn;
        assert!(
            item.move_to(RegionalVerifierSegmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
