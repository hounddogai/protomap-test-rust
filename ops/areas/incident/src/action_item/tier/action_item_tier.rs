use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a action item tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ActionItemTierStatus {
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

impl std::fmt::Display for ActionItemTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One action item tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemTier {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub limit: i64,
    pub grace_days: i64,
    pub created_at: i64,
    pub rotation_days: i64,
    pub tenant_id: String,
    pub attributes: BTreeMap<String, String>,
    pub status: ActionItemTierStatus,
}

impl ActionItemTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemTierStatus) -> bool {
        match self.status {
            ActionItemTierStatus::Planned => {
                match next {
                    ActionItemTierStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemTierStatus::Scheduled => {
                match next {
                    ActionItemTierStatus::Completed | ActionItemTierStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemTierStatus::InProgress => {
                return next == ActionItemTierStatus::Cancelled;
            }
            ActionItemTierStatus::Completed => {
                return next == ActionItemTierStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.grace_days as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemTier {
        ActionItemTier {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            limit: self.limit,
            grace_days: self.grace_days,
            created_at: self.created_at,
            rotation_days: self.rotation_days,
            tenant_id: self.tenant_id.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a action item tier in status planned.
pub fn new_action_item_tier(code: &str, label: &str) -> ActionItemTier {
    ActionItemTier {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        limit: 0,
        grace_days: 0,
        created_at: 0,
        rotation_days: 0,
        tenant_id: "".to_string(),
        attributes: BTreeMap::new(),
        status: ActionItemTierStatus::Planned,
    }
}
