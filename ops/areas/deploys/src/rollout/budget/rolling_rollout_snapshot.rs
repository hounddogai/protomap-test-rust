use super::*;

use acme_kit::text;

/// Where a rolling rollout snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRolloutSnapshotStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingRolloutSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingRolloutSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling rollout snapshot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutSnapshot {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub tenant_id: String,
    pub channel: String,
    pub status: RollingRolloutSnapshotStatus,
}

impl RollingRolloutSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRolloutSnapshotStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRolloutSnapshotStatus) -> bool {
        match self.status {
            RollingRolloutSnapshotStatus::Created => {
                match next {
                    RollingRolloutSnapshotStatus::Validated | RollingRolloutSnapshotStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRolloutSnapshotStatus::Validated => {
                return next == RollingRolloutSnapshotStatus::Retired;
            }
            RollingRolloutSnapshotStatus::Published => {
                match next {
                    RollingRolloutSnapshotStatus::Retired => {
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

    pub fn move_to(&mut self, next: RollingRolloutSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling rollout snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling rollout snapshot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> RollingRolloutSnapshot {
        RollingRolloutSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            tenant_id: self.tenant_id.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rolling rollout snapshot in status created.
pub fn new_rolling_rollout_snapshot(code: &str, label: &str) -> RollingRolloutSnapshot {
    RollingRolloutSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        tenant_id: "".to_string(),
        channel: "".to_string(),
        status: RollingRolloutSnapshotStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rolling_rollout_snapshot("code-610", "label-309");
        assert_eq!(item.status, RollingRolloutSnapshotStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_rolling_rollout_snapshot("code-610", "label-309");
        if item.move_to(RollingRolloutSnapshotStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRolloutSnapshotStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rolling_rollout_snapshot("code-610", "label-309");
        item.status = RollingRolloutSnapshotStatus::Retired;
        assert!(
            item.move_to(RollingRolloutSnapshotStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
