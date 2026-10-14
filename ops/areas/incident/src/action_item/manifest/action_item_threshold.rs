use super::*;

use acme_kit::text;

/// Where a action item threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ActionItemThresholdStatus {
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

impl std::fmt::Display for ActionItemThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item threshold as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemThreshold {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub external_ref: String,
    pub ratio: f64,
    pub status: ActionItemThresholdStatus,
}

impl ActionItemThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemThresholdStatus) -> bool {
        match self.status {
            ActionItemThresholdStatus::Idle => {
                match next {
                    ActionItemThresholdStatus::Draining | ActionItemThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemThresholdStatus::Warming => {
                match next {
                    ActionItemThresholdStatus::Draining | ActionItemThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemThresholdStatus::Ready => {
                match next {
                    ActionItemThresholdStatus::Draining | ActionItemThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemThresholdStatus::Draining => {
                return next == ActionItemThresholdStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.ratio * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemThreshold {
        ActionItemThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            external_ref: self.external_ref.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a action item threshold in status idle.
pub fn new_action_item_threshold(code: &str, title: &str) -> ActionItemThreshold {
    ActionItemThreshold {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        external_ref: "".to_string(),
        ratio: 0.0,
        status: ActionItemThresholdStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_action_item_threshold("code-24", "title-897");
        assert_eq!(item.status, ActionItemThresholdStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_action_item_threshold("code-24", "title-897");
        if item.move_to(ActionItemThresholdStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemThresholdStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_action_item_threshold("code-24", "title-897");
        item.status = ActionItemThresholdStatus::Stopped;
        assert!(
            item.move_to(ActionItemThresholdStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
