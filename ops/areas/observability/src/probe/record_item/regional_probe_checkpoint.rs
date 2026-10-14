use super::*;

use acme_kit::text;

/// Where a regional probe checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalProbeCheckpointStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RegionalProbeCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RegionalProbeCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a regional probe checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeCheckpoint {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub audience: String,
    pub ratio: f64,
    pub budget: f64,
    pub capacity: i64,
    pub active: bool,
    pub scope: String,
    pub status: RegionalProbeCheckpointStatus,
}

impl RegionalProbeCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalProbeCheckpointStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalProbeCheckpointStatus) -> bool {
        match self.status {
            RegionalProbeCheckpointStatus::Proposed => {
                return next == RegionalProbeCheckpointStatus::Rejected
                    || next == RegionalProbeCheckpointStatus::Withdrawn;
            }
            RegionalProbeCheckpointStatus::Approved => {
                return next == RegionalProbeCheckpointStatus::Rejected;
            }
            RegionalProbeCheckpointStatus::Rejected => {
                match next {
                    RegionalProbeCheckpointStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RegionalProbeCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional probe checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.capacity as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalProbeCheckpoint {
        RegionalProbeCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            pinned: self.pinned,
            audience: self.audience.clone(),
            ratio: self.ratio,
            budget: self.budget,
            capacity: self.capacity,
            active: self.active,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a regional probe checkpoint in status proposed.
pub fn new_regional_probe_checkpoint(code: &str, label: &str) -> RegionalProbeCheckpoint {
    RegionalProbeCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        pinned: false,
        audience: "".to_string(),
        ratio: 0.0,
        budget: 0.0,
        capacity: 0,
        active: false,
        scope: "".to_string(),
        status: RegionalProbeCheckpointStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_regional_probe_checkpoint("code-268", "label-43");
        assert_eq!(item.status, RegionalProbeCheckpointStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_regional_probe_checkpoint("code-268", "label-43");
        if item.move_to(RegionalProbeCheckpointStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalProbeCheckpointStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_regional_probe_checkpoint("code-268", "label-43");
        item.status = RegionalProbeCheckpointStatus::Withdrawn;
        assert!(
            item.move_to(RegionalProbeCheckpointStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
