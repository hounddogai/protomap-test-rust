use super::*;

use acme_kit::text;

/// Where a global overage snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalOverageSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalOverageSnapshotStatus {
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

impl std::fmt::Display for GlobalOverageSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global overage snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOverageSnapshot {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub timezone: String,
    pub cost: f64,
    pub status: GlobalOverageSnapshotStatus,
}

impl GlobalOverageSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalOverageSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalOverageSnapshotStatus) -> bool {
        match self.status {
            GlobalOverageSnapshotStatus::Idle => {
                return next == GlobalOverageSnapshotStatus::Ready || next == GlobalOverageSnapshotStatus::Stopped;
            }
            GlobalOverageSnapshotStatus::Warming => {
                return next == GlobalOverageSnapshotStatus::Ready;
            }
            GlobalOverageSnapshotStatus::Ready => {
                return next == GlobalOverageSnapshotStatus::Draining;
            }
            GlobalOverageSnapshotStatus::Draining => {
                match next {
                    GlobalOverageSnapshotStatus::Stopped => {
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

    pub fn move_to(&mut self, next: GlobalOverageSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global overage snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.cost + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalOverageSnapshot {
        GlobalOverageSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            timezone: self.timezone.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global overage snapshot in status idle.
pub fn new_global_overage_snapshot(code: &str, name: &str) -> GlobalOverageSnapshot {
    GlobalOverageSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        timezone: "".to_string(),
        cost: 0.0,
        status: GlobalOverageSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_overage_snapshot("code-47", "name-831");
        assert_eq!(item.status, GlobalOverageSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_global_overage_snapshot("code-47", "name-831");
        if item.move_to(GlobalOverageSnapshotStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalOverageSnapshotStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_overage_snapshot("code-47", "name-831");
        item.status = GlobalOverageSnapshotStatus::Stopped;
        assert!(
            item.move_to(GlobalOverageSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
