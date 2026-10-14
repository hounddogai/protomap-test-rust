use super::*;

use acme_kit::text;

/// Where a weighted rule snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRuleSnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedRuleSnapshotStatus {
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

impl std::fmt::Display for WeightedRuleSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted rule snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleSnapshot {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub ceiling: f64,
    pub maximum: f64,
    pub status: WeightedRuleSnapshotStatus,
}

impl WeightedRuleSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRuleSnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRuleSnapshotStatus) -> bool {
        match self.status {
            WeightedRuleSnapshotStatus::Idle => {
                return next == WeightedRuleSnapshotStatus::Draining;
            }
            WeightedRuleSnapshotStatus::Warming => {
                return next == WeightedRuleSnapshotStatus::Ready;
            }
            WeightedRuleSnapshotStatus::Ready => {
                return next == WeightedRuleSnapshotStatus::Draining;
            }
            WeightedRuleSnapshotStatus::Draining => {
                return next == WeightedRuleSnapshotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedRuleSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted rule snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted rule snapshot", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ratio - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRuleSnapshot {
        WeightedRuleSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            ceiling: self.ceiling,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted rule snapshot in status idle.
pub fn new_weighted_rule_snapshot(code: &str, title: &str) -> WeightedRuleSnapshot {
    WeightedRuleSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        ceiling: 0.0,
        maximum: 0.0,
        status: WeightedRuleSnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_rule_snapshot("code-621", "title-921");
        assert_eq!(item.status, WeightedRuleSnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_weighted_rule_snapshot("code-621", "title-921");
        if item.move_to(WeightedRuleSnapshotStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRuleSnapshotStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_rule_snapshot("code-621", "title-921");
        item.status = WeightedRuleSnapshotStatus::Stopped;
        assert!(
            item.move_to(WeightedRuleSnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
