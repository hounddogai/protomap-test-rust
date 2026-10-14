use super::*;

use acme_kit::text;

/// Where a action item entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ActionItemEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ActionItemEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemEntry {
    pub code: String,
    pub title: String,
    pub label: String,
    pub ends_at: i64,
    pub burst: i64,
    pub status: ActionItemEntryStatus,
}

impl ActionItemEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemEntryStatus) -> bool {
        match self.status {
            ActionItemEntryStatus::Idle => {
                return next == ActionItemEntryStatus::Draining || next == ActionItemEntryStatus::Stopped;
            }
            ActionItemEntryStatus::Warming => {
                match next {
                    ActionItemEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemEntryStatus::Ready => {
                match next {
                    ActionItemEntryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemEntryStatus::Draining => {
                return next == ActionItemEntryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "action item entry", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.burst as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemEntry {
        ActionItemEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a action item entry in status idle.
pub fn new_action_item_entry(code: &str, title: &str) -> ActionItemEntry {
    ActionItemEntry {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        ends_at: 0,
        burst: 0,
        status: ActionItemEntryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_action_item_entry("code-334", "title-143");
        assert_eq!(item.status, ActionItemEntryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_action_item_entry("code-334", "title-143");
        if item.move_to(ActionItemEntryStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemEntryStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_action_item_entry("code-334", "title-143");
        item.status = ActionItemEntryStatus::Stopped;
        assert!(
            item.move_to(ActionItemEntryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
