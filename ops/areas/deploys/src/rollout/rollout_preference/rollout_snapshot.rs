use super::*;

use acme_kit::text;

/// Where a rollout snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RolloutSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RolloutSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollout snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSnapshot {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub pinned: bool,
    pub priority: i64,
    pub discount: f64,
    pub status: RolloutSnapshotStatus,
}

impl RolloutSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutSnapshotStatus) -> bool {
        match self.status {
            RolloutSnapshotStatus::Created => {
                match next {
                    RolloutSnapshotStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutSnapshotStatus::Validated => {
                return next == RolloutSnapshotStatus::Published || next == RolloutSnapshotStatus::Retired;
            }
            RolloutSnapshotStatus::Published => {
                return next == RolloutSnapshotStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.quantity as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutSnapshot {
        RolloutSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            pinned: self.pinned,
            priority: self.priority,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollout snapshot in status created.
pub fn new_rollout_snapshot(code: &str, name: &str) -> RolloutSnapshot {
    RolloutSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        pinned: false,
        priority: 0,
        discount: 0.0,
        status: RolloutSnapshotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rollout_snapshot("code-15", "name-486");
        assert_eq!(item.status, RolloutSnapshotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_rollout_snapshot("code-15", "name-486");
        if item.move_to(RolloutSnapshotStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutSnapshotStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rollout_snapshot("code-15", "name-486");
        item.status = RolloutSnapshotStatus::Retired;
        assert!(
            item.move_to(RolloutSnapshotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
