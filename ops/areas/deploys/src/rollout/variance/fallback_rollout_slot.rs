use super::*;

use acme_kit::text;

/// Where a fallback rollout slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRolloutSlotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackRolloutSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackRolloutSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback rollout slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRolloutSlot {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub window_days: i64,
    pub channel: String,
    pub visible: bool,
    pub burst: i64,
    pub status: FallbackRolloutSlotStatus,
}

impl FallbackRolloutSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRolloutSlotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRolloutSlotStatus) -> bool {
        match self.status {
            FallbackRolloutSlotStatus::Created => {
                return next == FallbackRolloutSlotStatus::Published;
            }
            FallbackRolloutSlotStatus::Validated => {
                return next == FallbackRolloutSlotStatus::Published;
            }
            FallbackRolloutSlotStatus::Published => {
                return next == FallbackRolloutSlotStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackRolloutSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback rollout slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback rollout slot", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.window_days as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRolloutSlot {
        FallbackRolloutSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            window_days: self.window_days,
            channel: self.channel.clone(),
            visible: self.visible,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a fallback rollout slot in status created.
pub fn new_fallback_rollout_slot(code: &str, label: &str) -> FallbackRolloutSlot {
    FallbackRolloutSlot {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        window_days: 0,
        channel: "".to_string(),
        visible: false,
        burst: 0,
        status: FallbackRolloutSlotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_rollout_slot("code-839", "label-323");
        assert_eq!(item.status, FallbackRolloutSlotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_fallback_rollout_slot("code-839", "label-323");
        if item.move_to(FallbackRolloutSlotStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRolloutSlotStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_rollout_slot("code-839", "label-323");
        item.status = FallbackRolloutSlotStatus::Retired;
        assert!(
            item.move_to(FallbackRolloutSlotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
