use super::*;

use acme_kit::text;

/// Where a region pin cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionPinCycleStatus {
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

impl std::fmt::Display for RegionPinCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A region pin cycle as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinCycle {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub rotation_days: i64,
    pub rate: f64,
    pub status: RegionPinCycleStatus,
}

impl RegionPinCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinCycleStatus) -> bool {
        match self.status {
            RegionPinCycleStatus::Idle => {
                match next {
                    RegionPinCycleStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinCycleStatus::Warming => {
                return next == RegionPinCycleStatus::Draining;
            }
            RegionPinCycleStatus::Ready => {
                return next == RegionPinCycleStatus::Draining || next == RegionPinCycleStatus::Stopped;
            }
            RegionPinCycleStatus::Draining => {
                return next == RegionPinCycleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "region pin cycle", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.rotation_days as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinCycle {
        RegionPinCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            rotation_days: self.rotation_days,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a region pin cycle in status idle.
pub fn new_region_pin_cycle(code: &str, title: &str) -> RegionPinCycle {
    RegionPinCycle {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        rotation_days: 0,
        rate: 0.0,
        status: RegionPinCycleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_region_pin_cycle("code-496", "title-449");
        assert_eq!(item.status, RegionPinCycleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_region_pin_cycle("code-496", "title-449");
        if item.move_to(RegionPinCycleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinCycleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_region_pin_cycle("code-496", "title-449");
        item.status = RegionPinCycleStatus::Stopped;
        assert!(
            item.move_to(RegionPinCycleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
