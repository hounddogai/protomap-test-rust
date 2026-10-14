use super::*;

use acme_kit::text;

/// Where a scheduled action item adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledActionItemAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledActionItemAdjustmentStatus {
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

impl std::fmt::Display for ScheduledActionItemAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled action item adjustment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledActionItemAdjustment {
    pub code: String,
    pub title: String,
    pub country: String,
    pub external_ref: String,
    pub checksum: String,
    pub amount: f64,
    pub status: ScheduledActionItemAdjustmentStatus,
}

impl ScheduledActionItemAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledActionItemAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledActionItemAdjustmentStatus) -> bool {
        match self.status {
            ScheduledActionItemAdjustmentStatus::Idle => {
                return next == ScheduledActionItemAdjustmentStatus::Draining;
            }
            ScheduledActionItemAdjustmentStatus::Warming => {
                return next == ScheduledActionItemAdjustmentStatus::Draining
                    || next == ScheduledActionItemAdjustmentStatus::Stopped;
            }
            ScheduledActionItemAdjustmentStatus::Ready => {
                match next {
                    ScheduledActionItemAdjustmentStatus::Draining | ScheduledActionItemAdjustmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledActionItemAdjustmentStatus::Draining => {
                return next == ScheduledActionItemAdjustmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledActionItemAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled action item adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> ScheduledActionItemAdjustment {
        ScheduledActionItemAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            external_ref: self.external_ref.clone(),
            checksum: self.checksum.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a scheduled action item adjustment in status idle.
pub fn new_scheduled_action_item_adjustment(code: &str, title: &str) -> ScheduledActionItemAdjustment {
    ScheduledActionItemAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        external_ref: "".to_string(),
        checksum: "".to_string(),
        amount: 0.0,
        status: ScheduledActionItemAdjustmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_action_item_adjustment("code-156", "title-275");
        assert_eq!(item.status, ScheduledActionItemAdjustmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_scheduled_action_item_adjustment("code-156", "title-275");
        if item.move_to(ScheduledActionItemAdjustmentStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledActionItemAdjustmentStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_action_item_adjustment("code-156", "title-275");
        item.status = ScheduledActionItemAdjustmentStatus::Stopped;
        assert!(
            item.move_to(ScheduledActionItemAdjustmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
