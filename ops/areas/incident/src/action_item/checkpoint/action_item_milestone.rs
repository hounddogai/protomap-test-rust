use super::*;

use acme_kit::text;

/// Where a action item milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ActionItemMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ActionItemMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item milestone as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMilestone {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub capacity: i64,
    pub duration_minutes: i64,
    pub reviewer: String,
    pub channel: String,
    pub issuer: String,
    pub starts_at: i64,
    pub external_ref: String,
    pub retries: i64,
    pub status: ActionItemMilestoneStatus,
}

impl ActionItemMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemMilestoneStatus) -> bool {
        match self.status {
            ActionItemMilestoneStatus::Draft => {
                match next {
                    ActionItemMilestoneStatus::Active | ActionItemMilestoneStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemMilestoneStatus::Pending => {
                return next == ActionItemMilestoneStatus::Suspended || next == ActionItemMilestoneStatus::Closed;
            }
            ActionItemMilestoneStatus::Active => {
                return next == ActionItemMilestoneStatus::Suspended || next == ActionItemMilestoneStatus::Closed;
            }
            ActionItemMilestoneStatus::Suspended => {
                return next == ActionItemMilestoneStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.updated_at as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemMilestone {
        ActionItemMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            capacity: self.capacity,
            duration_minutes: self.duration_minutes,
            reviewer: self.reviewer.clone(),
            channel: self.channel.clone(),
            issuer: self.issuer.clone(),
            starts_at: self.starts_at,
            external_ref: self.external_ref.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a action item milestone in status draft.
pub fn new_action_item_milestone(code: &str, label: &str) -> ActionItemMilestone {
    ActionItemMilestone {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        capacity: 0,
        duration_minutes: 0,
        reviewer: "".to_string(),
        channel: "".to_string(),
        issuer: "".to_string(),
        starts_at: 0,
        external_ref: "".to_string(),
        retries: 0,
        status: ActionItemMilestoneStatus::Draft,
    }
}
