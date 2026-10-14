use super::*;

use acme_kit::text;

/// Where a legacy probe assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyProbeAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyProbeAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyProbeAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy probe assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeAssignment {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub duration_minutes: i64,
    pub name: String,
    pub region: String,
    pub rate: f64,
    pub floor_value: f64,
    pub locale: String,
    pub country: String,
    pub category: String,
    pub status: LegacyProbeAssignmentStatus,
}

impl LegacyProbeAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyProbeAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyProbeAssignmentStatus) -> bool {
        match self.status {
            LegacyProbeAssignmentStatus::Proposed => {
                return next == LegacyProbeAssignmentStatus::Withdrawn;
            }
            LegacyProbeAssignmentStatus::Approved => {
                match next {
                    LegacyProbeAssignmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyProbeAssignmentStatus::Rejected => {
                return next == LegacyProbeAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyProbeAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy probe assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.rate + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyProbeAssignment {
        LegacyProbeAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            duration_minutes: self.duration_minutes,
            name: self.name.clone(),
            region: self.region.clone(),
            rate: self.rate,
            floor_value: self.floor_value,
            locale: self.locale.clone(),
            country: self.country.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a legacy probe assignment in status proposed.
pub fn new_legacy_probe_assignment(code: &str, title: &str) -> LegacyProbeAssignment {
    LegacyProbeAssignment {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        duration_minutes: 0,
        name: "".to_string(),
        region: "".to_string(),
        rate: 0.0,
        floor_value: 0.0,
        locale: "".to_string(),
        country: "".to_string(),
        category: "".to_string(),
        status: LegacyProbeAssignmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_legacy_probe_assignment("code-565", "title-675");
        assert_eq!(item.status, LegacyProbeAssignmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_legacy_probe_assignment("code-565", "title-675");
        if item.move_to(LegacyProbeAssignmentStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyProbeAssignmentStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_legacy_probe_assignment("code-565", "title-675");
        item.status = LegacyProbeAssignmentStatus::Withdrawn;
        assert!(
            item.move_to(LegacyProbeAssignmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
