use super::*;

use acme_kit::text;

/// Where a plan window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanWindowStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PlanWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PlanWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanWindow {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub checksum: String,
    pub channel: String,
    pub cost: f64,
    pub window_days: i64,
    pub attempts: i64,
    pub pinned: bool,
    pub floor_value: f64,
    pub status: PlanWindowStatus,
}

impl PlanWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanWindowStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanWindowStatus) -> bool {
        match self.status {
            PlanWindowStatus::Proposed => {
                return next == PlanWindowStatus::Approved || next == PlanWindowStatus::Withdrawn;
            }
            PlanWindowStatus::Approved => {
                match next {
                    PlanWindowStatus::Rejected | PlanWindowStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanWindowStatus::Rejected => {
                return next == PlanWindowStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan window", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.floor_value + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanWindow {
        PlanWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            checksum: self.checksum.clone(),
            channel: self.channel.clone(),
            cost: self.cost,
            window_days: self.window_days,
            attempts: self.attempts,
            pinned: self.pinned,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a plan window in status proposed.
pub fn new_plan_window(code: &str, name: &str) -> PlanWindow {
    PlanWindow {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        checksum: "".to_string(),
        channel: "".to_string(),
        cost: 0.0,
        window_days: 0,
        attempts: 0,
        pinned: false,
        floor_value: 0.0,
        status: PlanWindowStatus::Proposed,
    }
}
