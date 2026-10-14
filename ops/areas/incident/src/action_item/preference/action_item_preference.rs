use super::*;

use acme_kit::text;

/// Where a action item preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ActionItemPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ActionItemPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPreference {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub active: bool,
    pub starts_at: i64,
    pub unit: String,
    pub status: ActionItemPreferenceStatus,
}

impl ActionItemPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemPreferenceStatus) -> bool {
        match self.status {
            ActionItemPreferenceStatus::Proposed => {
                return next == ActionItemPreferenceStatus::Withdrawn;
            }
            ActionItemPreferenceStatus::Approved => {
                return next == ActionItemPreferenceStatus::Rejected || next == ActionItemPreferenceStatus::Withdrawn;
            }
            ActionItemPreferenceStatus::Rejected => {
                return next == ActionItemPreferenceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item preference", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> ActionItemPreference {
        ActionItemPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            active: self.active,
            starts_at: self.starts_at,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a action item preference in status proposed.
pub fn new_action_item_preference(code: &str, title: &str) -> ActionItemPreference {
    ActionItemPreference {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        active: false,
        starts_at: 0,
        unit: "".to_string(),
        status: ActionItemPreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_action_item_preference("code-572", "title-999");
        assert_eq!(item.status, ActionItemPreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_action_item_preference("code-572", "title-999");
        if item.move_to(ActionItemPreferenceStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemPreferenceStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_action_item_preference("code-572", "title-999");
        item.status = ActionItemPreferenceStatus::Withdrawn;
        assert!(
            item.move_to(ActionItemPreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
