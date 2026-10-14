use super::*;

use acme_kit::text;

/// Where a tiered alert slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredAlertSlotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredAlertSlotStatus {
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

impl std::fmt::Display for TieredAlertSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered alert slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertSlot {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub checksum: String,
    pub floor_value: f64,
    pub version: i64,
    pub status: TieredAlertSlotStatus,
}

impl TieredAlertSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredAlertSlotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredAlertSlotStatus) -> bool {
        match self.status {
            TieredAlertSlotStatus::Idle => {
                match next {
                    TieredAlertSlotStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAlertSlotStatus::Warming => {
                match next {
                    TieredAlertSlotStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredAlertSlotStatus::Ready => {
                return next == TieredAlertSlotStatus::Draining || next == TieredAlertSlotStatus::Stopped;
            }
            TieredAlertSlotStatus::Draining => {
                match next {
                    TieredAlertSlotStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TieredAlertSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered alert slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered alert slot", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.version as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredAlertSlot {
        TieredAlertSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            checksum: self.checksum.clone(),
            floor_value: self.floor_value,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered alert slot in status idle.
pub fn new_tiered_alert_slot(code: &str, label: &str) -> TieredAlertSlot {
    TieredAlertSlot {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        checksum: "".to_string(),
        floor_value: 0.0,
        version: 0,
        status: TieredAlertSlotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_alert_slot("code-718", "label-338");
        assert_eq!(item.status, TieredAlertSlotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_tiered_alert_slot("code-718", "label-338");
        if item.move_to(TieredAlertSlotStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredAlertSlotStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_alert_slot("code-718", "label-338");
        item.status = TieredAlertSlotStatus::Stopped;
        assert!(
            item.move_to(TieredAlertSlotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
