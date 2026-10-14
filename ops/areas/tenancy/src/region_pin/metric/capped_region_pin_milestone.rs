use super::*;

use acme_kit::text;

/// Where a capped region pin milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedRegionPinMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedRegionPinMilestoneStatus {
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

impl std::fmt::Display for CappedRegionPinMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped region pin milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRegionPinMilestone {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub amount: f64,
    pub priority: i64,
    pub window_days: i64,
    pub status: CappedRegionPinMilestoneStatus,
}

impl CappedRegionPinMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedRegionPinMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedRegionPinMilestoneStatus) -> bool {
        match self.status {
            CappedRegionPinMilestoneStatus::Idle => {
                match next {
                    CappedRegionPinMilestoneStatus::Draining | CappedRegionPinMilestoneStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRegionPinMilestoneStatus::Warming => {
                match next {
                    CappedRegionPinMilestoneStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRegionPinMilestoneStatus::Ready => {
                match next {
                    CappedRegionPinMilestoneStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedRegionPinMilestoneStatus::Draining => {
                return next == CappedRegionPinMilestoneStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedRegionPinMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped region pin milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.window_days as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedRegionPinMilestone {
        CappedRegionPinMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            amount: self.amount,
            priority: self.priority,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped region pin milestone in status idle.
pub fn new_capped_region_pin_milestone(code: &str, title: &str) -> CappedRegionPinMilestone {
    CappedRegionPinMilestone {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        amount: 0.0,
        priority: 0,
        window_days: 0,
        status: CappedRegionPinMilestoneStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_capped_region_pin_milestone("code-273", "title-261");
        assert_eq!(item.status, CappedRegionPinMilestoneStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_capped_region_pin_milestone("code-273", "title-261");
        if item.move_to(CappedRegionPinMilestoneStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedRegionPinMilestoneStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_capped_region_pin_milestone("code-273", "title-261");
        item.status = CappedRegionPinMilestoneStatus::Stopped;
        assert!(
            item.move_to(CappedRegionPinMilestoneStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
