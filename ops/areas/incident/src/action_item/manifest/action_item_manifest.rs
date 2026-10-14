use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a action item manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ActionItemManifestStatus {
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

impl std::fmt::Display for ActionItemManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item manifest as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemManifest {
    pub code: String,
    pub title: String,
    pub label: String,
    pub rate: f64,
    pub channel: String,
    pub attributes: BTreeMap<String, String>,
    pub minimum: f64,
    pub limit: i64,
    pub floor_value: f64,
    pub pinned: bool,
    pub status: ActionItemManifestStatus,
}

impl ActionItemManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemManifestStatus) -> bool {
        match self.status {
            ActionItemManifestStatus::Planned => {
                match next {
                    ActionItemManifestStatus::Scheduled | ActionItemManifestStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemManifestStatus::Scheduled => {
                match next {
                    ActionItemManifestStatus::InProgress | ActionItemManifestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemManifestStatus::InProgress => {
                match next {
                    ActionItemManifestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemManifestStatus::Completed => {
                match next {
                    ActionItemManifestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ActionItemManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn adjusted_total(&self) -> f64 {
        ((self.floor_value + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemManifest {
        ActionItemManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            rate: self.rate,
            channel: self.channel.clone(),
            attributes: self.attributes.clone(),
            minimum: self.minimum,
            limit: self.limit,
            floor_value: self.floor_value,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a action item manifest in status planned.
pub fn new_action_item_manifest(code: &str, title: &str) -> ActionItemManifest {
    ActionItemManifest {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        rate: 0.0,
        channel: "".to_string(),
        attributes: BTreeMap::new(),
        minimum: 0.0,
        limit: 0,
        floor_value: 0.0,
        pinned: false,
        status: ActionItemManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_action_item_manifest("code-376", "title-618");
        assert_eq!(item.status, ActionItemManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_action_item_manifest("code-376", "title-618");
        if item.move_to(ActionItemManifestStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemManifestStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_action_item_manifest("code-376", "title-618");
        item.status = ActionItemManifestStatus::Cancelled;
        assert!(
            item.move_to(ActionItemManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
