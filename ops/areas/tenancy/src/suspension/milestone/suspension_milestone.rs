use super::*;

use acme_kit::text;

/// Where a suspension milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionMilestoneStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SuspensionMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SuspensionMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionMilestone {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub burst: i64,
    pub phone: String,
    pub capacity: i64,
    pub cost: f64,
    pub status: SuspensionMilestoneStatus,
}

impl SuspensionMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionMilestoneStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionMilestoneStatus) -> bool {
        match self.status {
            SuspensionMilestoneStatus::Requested => {
                return next == SuspensionMilestoneStatus::Revoked;
            }
            SuspensionMilestoneStatus::Granted => {
                match next {
                    SuspensionMilestoneStatus::Revoked | SuspensionMilestoneStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionMilestoneStatus::Revoked => {
                return next == SuspensionMilestoneStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "suspension milestone", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        (self.cost * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionMilestone {
        SuspensionMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            burst: self.burst,
            phone: self.phone.clone(),
            capacity: self.capacity,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a suspension milestone in status requested.
pub fn new_suspension_milestone(code: &str, label: &str) -> SuspensionMilestone {
    SuspensionMilestone {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        burst: 0,
        phone: "".to_string(),
        capacity: 0,
        cost: 0.0,
        status: SuspensionMilestoneStatus::Requested,
    }
}
