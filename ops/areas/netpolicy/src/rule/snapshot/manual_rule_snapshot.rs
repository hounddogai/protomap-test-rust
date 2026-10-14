use super::*;

use acme_kit::text;

/// Where a manual rule snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRuleSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualRuleSnapshotStatus {
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

impl std::fmt::Display for ManualRuleSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual rule snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleSnapshot {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub rotation_days: i64,
    pub weight: f64,
    pub status: ManualRuleSnapshotStatus,
}

impl ManualRuleSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRuleSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRuleSnapshotStatus) -> bool {
        match self.status {
            ManualRuleSnapshotStatus::Idle => {
                return next == ManualRuleSnapshotStatus::Warming || next == ManualRuleSnapshotStatus::Draining;
            }
            ManualRuleSnapshotStatus::Warming => {
                match next {
                    ManualRuleSnapshotStatus::Draining | ManualRuleSnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRuleSnapshotStatus::Ready => {
                return next == ManualRuleSnapshotStatus::Draining || next == ManualRuleSnapshotStatus::Stopped;
            }
            ManualRuleSnapshotStatus::Draining => {
                return next == ManualRuleSnapshotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualRuleSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual rule snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.weight + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRuleSnapshot {
        ManualRuleSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            rotation_days: self.rotation_days,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual rule snapshot in status idle.
pub fn new_manual_rule_snapshot(code: &str, name: &str) -> ManualRuleSnapshot {
    ManualRuleSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        rotation_days: 0,
        weight: 0.0,
        status: ManualRuleSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_rule_snapshot("code-104", "name-104");
        assert_eq!(item.status, ManualRuleSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_manual_rule_snapshot("code-104", "name-104");
        if item.move_to(ManualRuleSnapshotStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRuleSnapshotStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_rule_snapshot("code-104", "name-104");
        item.status = ManualRuleSnapshotStatus::Stopped;
        assert!(
            item.move_to(ManualRuleSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
