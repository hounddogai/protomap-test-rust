use super::*;

use acme_kit::text;

/// Where a primary rollout batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRolloutBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryRolloutBatchStatus {
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

impl std::fmt::Display for PrimaryRolloutBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary rollout batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutBatch {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub port: i64,
    pub rate: f64,
    pub sequence: i64,
    pub unit: String,
    pub source: String,
    pub status: PrimaryRolloutBatchStatus,
}

impl PrimaryRolloutBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRolloutBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRolloutBatchStatus) -> bool {
        match self.status {
            PrimaryRolloutBatchStatus::Idle => {
                return next == PrimaryRolloutBatchStatus::Stopped;
            }
            PrimaryRolloutBatchStatus::Warming => {
                return next == PrimaryRolloutBatchStatus::Ready || next == PrimaryRolloutBatchStatus::Draining;
            }
            PrimaryRolloutBatchStatus::Ready => {
                return next == PrimaryRolloutBatchStatus::Draining;
            }
            PrimaryRolloutBatchStatus::Draining => {
                return next == PrimaryRolloutBatchStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryRolloutBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rollout batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rate + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRolloutBatch {
        PrimaryRolloutBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            port: self.port,
            rate: self.rate,
            sequence: self.sequence,
            unit: self.unit.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary rollout batch in status idle.
pub fn new_primary_rollout_batch(code: &str, name: &str) -> PrimaryRolloutBatch {
    PrimaryRolloutBatch {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        port: 0,
        rate: 0.0,
        sequence: 0,
        unit: "".to_string(),
        source: "".to_string(),
        status: PrimaryRolloutBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_primary_rollout_batch("code-754", "name-800");
        assert_eq!(item.status, PrimaryRolloutBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_primary_rollout_batch("code-754", "name-800");
        if item.move_to(PrimaryRolloutBatchStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRolloutBatchStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_primary_rollout_batch("code-754", "name-800");
        item.status = PrimaryRolloutBatchStatus::Stopped;
        assert!(
            item.move_to(PrimaryRolloutBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
