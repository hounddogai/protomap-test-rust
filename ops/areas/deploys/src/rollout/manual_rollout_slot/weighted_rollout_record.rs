use super::*;

use acme_kit::text;

/// Where a weighted rollout record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedRolloutRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedRolloutRecordStatus {
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

impl std::fmt::Display for WeightedRolloutRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted rollout record, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRolloutRecord {
    pub code: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub updated_at: i64,
    pub audience: String,
    pub ratio: f64,
    pub status: WeightedRolloutRecordStatus,
}

impl WeightedRolloutRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedRolloutRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedRolloutRecordStatus) -> bool {
        match self.status {
            WeightedRolloutRecordStatus::New => {
                return next == WeightedRolloutRecordStatus::Queued || next == WeightedRolloutRecordStatus::Failed;
            }
            WeightedRolloutRecordStatus::Queued => {
                return next == WeightedRolloutRecordStatus::Running;
            }
            WeightedRolloutRecordStatus::Running => {
                match next {
                    WeightedRolloutRecordStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedRolloutRecordStatus::Done => {
                match next {
                    WeightedRolloutRecordStatus::Failed => {
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

    pub fn move_to(&mut self, next: WeightedRolloutRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted rollout record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted rollout record", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.ratio * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedRolloutRecord {
        WeightedRolloutRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            updated_at: self.updated_at,
            audience: self.audience.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted rollout record in status new.
pub fn new_weighted_rollout_record(code: &str, name: &str) -> WeightedRolloutRecord {
    WeightedRolloutRecord {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        description: "".to_string(),
        updated_at: 0,
        audience: "".to_string(),
        ratio: 0.0,
        status: WeightedRolloutRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_weighted_rollout_record("code-694", "name-766");
        assert_eq!(item.status, WeightedRolloutRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_weighted_rollout_record("code-694", "name-766");
        if item.move_to(WeightedRolloutRecordStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedRolloutRecordStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_weighted_rollout_record("code-694", "name-766");
        item.status = WeightedRolloutRecordStatus::Failed;
        assert!(
            item.move_to(WeightedRolloutRecordStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
