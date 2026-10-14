use super::*;

use acme_kit::text;

/// Where a usage slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageSlotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl UsageSlotStatus {
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

impl std::fmt::Display for UsageSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSlot {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub discount: f64,
    pub ttl_seconds: i64,
    pub maximum: f64,
    pub minimum: f64,
    pub status: UsageSlotStatus,
}

impl UsageSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageSlotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageSlotStatus) -> bool {
        match self.status {
            UsageSlotStatus::Idle => {
                return next == UsageSlotStatus::Ready;
            }
            UsageSlotStatus::Warming => {
                match next {
                    UsageSlotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageSlotStatus::Ready => {
                return next == UsageSlotStatus::Draining || next == UsageSlotStatus::Stopped;
            }
            UsageSlotStatus::Draining => {
                return next == UsageSlotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "usage slot", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.maximum - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageSlot {
        UsageSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            discount: self.discount,
            ttl_seconds: self.ttl_seconds,
            maximum: self.maximum,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a usage slot in status idle.
pub fn new_usage_slot(code: &str, title: &str) -> UsageSlot {
    UsageSlot {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        discount: 0.0,
        ttl_seconds: 0,
        maximum: 0.0,
        minimum: 0.0,
        status: UsageSlotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_usage_slot("code-873", "title-948");
        assert_eq!(item.status, UsageSlotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_usage_slot("code-873", "title-948");
        if item.move_to(UsageSlotStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageSlotStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_usage_slot("code-873", "title-948");
        item.status = UsageSlotStatus::Stopped;
        assert!(item.move_to(UsageSlotStatus::Idle).is_err(), "expected move to fail");
    }
}
