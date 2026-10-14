use super::*;

use acme_kit::text;

/// Where a action item snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemSnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ActionItemSnapshotStatus {
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

impl std::fmt::Display for ActionItemSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSnapshot {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub weight: f64,
    pub burst: i64,
    pub tenant_id: String,
    pub sequence: i64,
    pub status: ActionItemSnapshotStatus,
}

impl ActionItemSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemSnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemSnapshotStatus) -> bool {
        match self.status {
            ActionItemSnapshotStatus::Draft => {
                match next {
                    ActionItemSnapshotStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemSnapshotStatus::Pending => {
                return next == ActionItemSnapshotStatus::Closed;
            }
            ActionItemSnapshotStatus::Active => {
                match next {
                    ActionItemSnapshotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemSnapshotStatus::Suspended => {
                match next {
                    ActionItemSnapshotStatus::Closed => {
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

    pub fn move_to(&mut self, next: ActionItemSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.sequence as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemSnapshot {
        ActionItemSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            weight: self.weight,
            burst: self.burst,
            tenant_id: self.tenant_id.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a action item snapshot in status draft.
pub fn new_action_item_snapshot(code: &str, name: &str) -> ActionItemSnapshot {
    ActionItemSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        weight: 0.0,
        burst: 0,
        tenant_id: "".to_string(),
        sequence: 0,
        status: ActionItemSnapshotStatus::Draft,
    }
}
