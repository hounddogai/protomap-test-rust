use super::*;

use acme_kit::text;

/// Where a rollout adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RolloutAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RolloutAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAdjustment {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub cluster: String,
    pub visible: bool,
    pub quota_units: i64,
    pub duration_minutes: i64,
    pub source: String,
    pub attempts: i64,
    pub port: i64,
    pub cipher: String,
    pub status: RolloutAdjustmentStatus,
}

impl RolloutAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutAdjustmentStatus) -> bool {
        match self.status {
            RolloutAdjustmentStatus::New => {
                return next == RolloutAdjustmentStatus::Queued || next == RolloutAdjustmentStatus::Done;
            }
            RolloutAdjustmentStatus::Queued => {
                return next == RolloutAdjustmentStatus::Done;
            }
            RolloutAdjustmentStatus::Running => {
                return next == RolloutAdjustmentStatus::Done || next == RolloutAdjustmentStatus::Failed;
            }
            RolloutAdjustmentStatus::Done => {
                match next {
                    RolloutAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: RolloutAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout adjustment", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.port as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutAdjustment {
        RolloutAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            cluster: self.cluster.clone(),
            visible: self.visible,
            quota_units: self.quota_units,
            duration_minutes: self.duration_minutes,
            source: self.source.clone(),
            attempts: self.attempts,
            port: self.port,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rollout adjustment in status new.
pub fn new_rollout_adjustment(code: &str, name: &str) -> RolloutAdjustment {
    RolloutAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        cluster: "".to_string(),
        visible: false,
        quota_units: 0,
        duration_minutes: 0,
        source: "".to_string(),
        attempts: 0,
        port: 0,
        cipher: "".to_string(),
        status: RolloutAdjustmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollout_adjustment("code-858", "name-696");
        assert_eq!(item.status, RolloutAdjustmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_rollout_adjustment("code-858", "name-696");
        if item.move_to(RolloutAdjustmentStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutAdjustmentStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollout_adjustment("code-858", "name-696");
        item.status = RolloutAdjustmentStatus::Failed;
        assert!(
            item.move_to(RolloutAdjustmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
