use super::*;

use acme_kit::text;

/// Where a strict dashboard slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictDashboardSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictDashboardSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for StrictDashboardSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict dashboard slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardSlot {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub namespace: String,
    pub unit: String,
    pub revision: i64,
    pub rotation_days: i64,
    pub checksum: String,
    pub quantity: i64,
    pub budget: f64,
    pub maximum: f64,
    pub status: StrictDashboardSlotStatus,
}

impl StrictDashboardSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictDashboardSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictDashboardSlotStatus) -> bool {
        match self.status {
            StrictDashboardSlotStatus::Planned => {
                return next == StrictDashboardSlotStatus::Cancelled;
            }
            StrictDashboardSlotStatus::Scheduled => {
                match next {
                    StrictDashboardSlotStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDashboardSlotStatus::InProgress => {
                return next == StrictDashboardSlotStatus::Completed || next == StrictDashboardSlotStatus::Cancelled;
            }
            StrictDashboardSlotStatus::Completed => {
                match next {
                    StrictDashboardSlotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: StrictDashboardSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict dashboard slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict dashboard slot", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quantity as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictDashboardSlot {
        StrictDashboardSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            namespace: self.namespace.clone(),
            unit: self.unit.clone(),
            revision: self.revision,
            rotation_days: self.rotation_days,
            checksum: self.checksum.clone(),
            quantity: self.quantity,
            budget: self.budget,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a strict dashboard slot in status planned.
pub fn new_strict_dashboard_slot(code: &str, name: &str) -> StrictDashboardSlot {
    StrictDashboardSlot {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        namespace: "".to_string(),
        unit: "".to_string(),
        revision: 0,
        rotation_days: 0,
        checksum: "".to_string(),
        quantity: 0,
        budget: 0.0,
        maximum: 0.0,
        status: StrictDashboardSlotStatus::Planned,
    }
}
