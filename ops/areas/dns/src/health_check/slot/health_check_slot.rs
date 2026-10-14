use super::*;

use acme_kit::text;

/// Where a health check slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckSlotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl HealthCheckSlotStatus {
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

impl std::fmt::Display for HealthCheckSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a health check slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckSlot {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub notes: String,
    pub budget: f64,
    pub status: HealthCheckSlotStatus,
}

impl HealthCheckSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckSlotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckSlotStatus) -> bool {
        match self.status {
            HealthCheckSlotStatus::Idle => {
                return next == HealthCheckSlotStatus::Stopped;
            }
            HealthCheckSlotStatus::Warming => {
                return next == HealthCheckSlotStatus::Ready || next == HealthCheckSlotStatus::Stopped;
            }
            HealthCheckSlotStatus::Ready => {
                match next {
                    HealthCheckSlotStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckSlotStatus::Draining => {
                match next {
                    HealthCheckSlotStatus::Stopped => {
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

    pub fn move_to(&mut self, next: HealthCheckSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "health check slot", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.grace_days as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckSlot {
        HealthCheckSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            notes: self.notes.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a health check slot in status idle.
pub fn new_health_check_slot(code: &str, title: &str) -> HealthCheckSlot {
    HealthCheckSlot {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        notes: "".to_string(),
        budget: 0.0,
        status: HealthCheckSlotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_health_check_slot("code-854", "title-739");
        assert_eq!(item.status, HealthCheckSlotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_health_check_slot("code-854", "title-739");
        if item.move_to(HealthCheckSlotStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckSlotStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_health_check_slot("code-854", "title-739");
        item.status = HealthCheckSlotStatus::Stopped;
        assert!(
            item.move_to(HealthCheckSlotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
