use super::*;

use acme_kit::text;

/// Where a rollout checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RolloutCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RolloutCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout checkpoint as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCheckpoint {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub budget: f64,
    pub source: String,
    pub ends_at: i64,
    pub starts_at: i64,
    pub reviewer: String,
    pub category: String,
    pub status: RolloutCheckpointStatus,
}

impl RolloutCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutCheckpointStatus) -> bool {
        match self.status {
            RolloutCheckpointStatus::Created => {
                match next {
                    RolloutCheckpointStatus::Published | RolloutCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutCheckpointStatus::Validated => {
                return next == RolloutCheckpointStatus::Published || next == RolloutCheckpointStatus::Retired;
            }
            RolloutCheckpointStatus::Published => {
                match next {
                    RolloutCheckpointStatus::Retired => {
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

    pub fn move_to(&mut self, next: RolloutCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.starts_at as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutCheckpoint {
        RolloutCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            budget: self.budget,
            source: self.source.clone(),
            ends_at: self.ends_at,
            starts_at: self.starts_at,
            reviewer: self.reviewer.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rollout checkpoint in status created.
pub fn new_rollout_checkpoint(code: &str, label: &str) -> RolloutCheckpoint {
    RolloutCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        budget: 0.0,
        source: "".to_string(),
        ends_at: 0,
        starts_at: 0,
        reviewer: "".to_string(),
        category: "".to_string(),
        status: RolloutCheckpointStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rollout_checkpoint("code-645", "label-743");
        assert_eq!(item.status, RolloutCheckpointStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_rollout_checkpoint("code-645", "label-743");
        if item.move_to(RolloutCheckpointStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutCheckpointStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rollout_checkpoint("code-645", "label-743");
        item.status = RolloutCheckpointStatus::Retired;
        assert!(
            item.move_to(RolloutCheckpointStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
