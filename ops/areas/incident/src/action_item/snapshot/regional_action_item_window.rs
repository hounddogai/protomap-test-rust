use super::*;

use acme_kit::text;

/// Where a regional action item window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalActionItemWindowStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalActionItemWindowStatus {
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

impl std::fmt::Display for RegionalActionItemWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional action item window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemWindow {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub channel: String,
    pub port: i64,
    pub status: RegionalActionItemWindowStatus,
}

impl RegionalActionItemWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalActionItemWindowStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalActionItemWindowStatus) -> bool {
        match self.status {
            RegionalActionItemWindowStatus::Planned => {
                return next == RegionalActionItemWindowStatus::Completed;
            }
            RegionalActionItemWindowStatus::Scheduled => {
                return next == RegionalActionItemWindowStatus::Completed
                    || next == RegionalActionItemWindowStatus::Cancelled;
            }
            RegionalActionItemWindowStatus::InProgress => {
                match next {
                    RegionalActionItemWindowStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalActionItemWindowStatus::Completed => {
                return next == RegionalActionItemWindowStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalActionItemWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional action item window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional action item window", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> RegionalActionItemWindow {
        RegionalActionItemWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            channel: self.channel.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a regional action item window in status planned.
pub fn new_regional_action_item_window(code: &str, label: &str) -> RegionalActionItemWindow {
    RegionalActionItemWindow {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        channel: "".to_string(),
        port: 0,
        status: RegionalActionItemWindowStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_regional_action_item_window("code-250", "label-91");
        assert_eq!(item.status, RegionalActionItemWindowStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_regional_action_item_window("code-250", "label-91");
        if item.move_to(RegionalActionItemWindowStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalActionItemWindowStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_regional_action_item_window("code-250", "label-91");
        item.status = RegionalActionItemWindowStatus::Cancelled;
        assert!(
            item.move_to(RegionalActionItemWindowStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
