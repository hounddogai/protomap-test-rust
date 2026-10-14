use super::*;

use acme_kit::text;

/// Where a seasonal canary checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalCanaryCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalCanaryCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SeasonalCanaryCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal canary checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryCheckpoint {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub ratio: f64,
    pub sequence: i64,
    pub created_at: i64,
    pub timezone: String,
    pub window_days: i64,
    pub status: SeasonalCanaryCheckpointStatus,
}

impl SeasonalCanaryCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalCanaryCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalCanaryCheckpointStatus) -> bool {
        match self.status {
            SeasonalCanaryCheckpointStatus::Planned => {
                match next {
                    SeasonalCanaryCheckpointStatus::Scheduled | SeasonalCanaryCheckpointStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCanaryCheckpointStatus::Scheduled => {
                match next {
                    SeasonalCanaryCheckpointStatus::InProgress | SeasonalCanaryCheckpointStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalCanaryCheckpointStatus::InProgress => {
                return next == SeasonalCanaryCheckpointStatus::Cancelled;
            }
            SeasonalCanaryCheckpointStatus::Completed => {
                return next == SeasonalCanaryCheckpointStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalCanaryCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal canary checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal canary checkpoint", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.window_days as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalCanaryCheckpoint {
        SeasonalCanaryCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            ratio: self.ratio,
            sequence: self.sequence,
            created_at: self.created_at,
            timezone: self.timezone.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal canary checkpoint in status planned.
pub fn new_seasonal_canary_checkpoint(code: &str, name: &str) -> SeasonalCanaryCheckpoint {
    SeasonalCanaryCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        ratio: 0.0,
        sequence: 0,
        created_at: 0,
        timezone: "".to_string(),
        window_days: 0,
        status: SeasonalCanaryCheckpointStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_seasonal_canary_checkpoint("code-209", "name-489");
        assert_eq!(item.status, SeasonalCanaryCheckpointStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_seasonal_canary_checkpoint("code-209", "name-489");
        if item.move_to(SeasonalCanaryCheckpointStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalCanaryCheckpointStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_seasonal_canary_checkpoint("code-209", "name-489");
        item.status = SeasonalCanaryCheckpointStatus::Cancelled;
        assert!(
            item.move_to(SeasonalCanaryCheckpointStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
