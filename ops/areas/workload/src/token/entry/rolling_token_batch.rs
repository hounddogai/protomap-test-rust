use super::*;

use acme_kit::text;

/// Where a rolling token batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTokenBatchStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingTokenBatchStatus {
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

impl std::fmt::Display for RollingTokenBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling token batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenBatch {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub audience: String,
    pub unit: String,
    pub external_ref: String,
    pub status: RollingTokenBatchStatus,
}

impl RollingTokenBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTokenBatchStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTokenBatchStatus) -> bool {
        match self.status {
            RollingTokenBatchStatus::Planned => {
                return next == RollingTokenBatchStatus::Scheduled;
            }
            RollingTokenBatchStatus::Scheduled => {
                match next {
                    RollingTokenBatchStatus::InProgress | RollingTokenBatchStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTokenBatchStatus::InProgress => {
                match next {
                    RollingTokenBatchStatus::Completed | RollingTokenBatchStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTokenBatchStatus::Completed => {
                match next {
                    RollingTokenBatchStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollingTokenBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling token batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> RollingTokenBatch {
        RollingTokenBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            audience: self.audience.clone(),
            unit: self.unit.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rolling token batch in status planned.
pub fn new_rolling_token_batch(code: &str, name: &str) -> RollingTokenBatch {
    RollingTokenBatch {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        audience: "".to_string(),
        unit: "".to_string(),
        external_ref: "".to_string(),
        status: RollingTokenBatchStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_token_batch("code-992", "name-991");
        assert_eq!(item.status, RollingTokenBatchStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rolling_token_batch("code-992", "name-991");
        if item.move_to(RollingTokenBatchStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingTokenBatchStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_token_batch("code-992", "name-991");
        item.status = RollingTokenBatchStatus::Cancelled;
        assert!(
            item.move_to(RollingTokenBatchStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
