use super::*;

use acme_kit::text;

/// Where a strict alert cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAlertCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictAlertCycleStatus {
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

impl std::fmt::Display for StrictAlertCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict alert cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertCycle {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub category: String,
    pub margin: f64,
    pub score: f64,
    pub revision: i64,
    pub minimum: f64,
    pub status: StrictAlertCycleStatus,
}

impl StrictAlertCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAlertCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAlertCycleStatus) -> bool {
        match self.status {
            StrictAlertCycleStatus::Idle => {
                match next {
                    StrictAlertCycleStatus::Warming | StrictAlertCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAlertCycleStatus::Warming => {
                match next {
                    StrictAlertCycleStatus::Draining | StrictAlertCycleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAlertCycleStatus::Ready => {
                return next == StrictAlertCycleStatus::Draining;
            }
            StrictAlertCycleStatus::Draining => {
                match next {
                    StrictAlertCycleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: StrictAlertCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict alert cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.margin - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictAlertCycle {
        StrictAlertCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            category: self.category.clone(),
            margin: self.margin,
            score: self.score,
            revision: self.revision,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict alert cycle in status idle.
pub fn new_strict_alert_cycle(code: &str, name: &str) -> StrictAlertCycle {
    StrictAlertCycle {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        category: "".to_string(),
        margin: 0.0,
        score: 0.0,
        revision: 0,
        minimum: 0.0,
        status: StrictAlertCycleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_alert_cycle("code-571", "name-146");
        assert_eq!(item.status, StrictAlertCycleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_strict_alert_cycle("code-571", "name-146");
        if item.move_to(StrictAlertCycleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictAlertCycleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_alert_cycle("code-571", "name-146");
        item.status = StrictAlertCycleStatus::Stopped;
        assert!(
            item.move_to(StrictAlertCycleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
