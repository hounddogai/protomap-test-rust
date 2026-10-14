use super::*;

use acme_kit::text;

/// Where a rule snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleSnapshotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RuleSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RuleSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule snapshot as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSnapshot {
    pub code: String,
    pub label: String,
    pub capacity: i64,
    pub source: String,
    pub visible: bool,
    pub floor_value: f64,
    pub status: RuleSnapshotStatus,
}

impl RuleSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleSnapshotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleSnapshotStatus) -> bool {
        match self.status {
            RuleSnapshotStatus::Proposed => {
                match next {
                    RuleSnapshotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleSnapshotStatus::Approved => {
                match next {
                    RuleSnapshotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleSnapshotStatus::Rejected => {
                return next == RuleSnapshotStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rule snapshot", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.floor_value - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleSnapshot {
        RuleSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            source: self.source.clone(),
            visible: self.visible,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rule snapshot in status proposed.
pub fn new_rule_snapshot(code: &str, label: &str) -> RuleSnapshot {
    RuleSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        capacity: 0,
        source: "".to_string(),
        visible: false,
        floor_value: 0.0,
        status: RuleSnapshotStatus::Proposed,
    }
}
