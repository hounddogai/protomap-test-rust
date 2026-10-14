use super::*;

use acme_kit::text;

/// Where a retention milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RetentionMilestoneStatus {
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

impl std::fmt::Display for RetentionMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionMilestone {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub revision: i64,
    pub version: i64,
    pub rotation_days: i64,
    pub amount: f64,
    pub owner_id: String,
    pub currency: String,
    pub tags: Vec<String>,
    pub quota_units: i64,
    pub status: RetentionMilestoneStatus,
}

impl RetentionMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionMilestoneStatus) -> bool {
        match self.status {
            RetentionMilestoneStatus::New => {
                return next == RetentionMilestoneStatus::Running;
            }
            RetentionMilestoneStatus::Queued => {
                match next {
                    RetentionMilestoneStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionMilestoneStatus::Running => {
                return next == RetentionMilestoneStatus::Done || next == RetentionMilestoneStatus::Failed;
            }
            RetentionMilestoneStatus::Done => {
                return next == RetentionMilestoneStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RetentionMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.rotation_days as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionMilestone {
        RetentionMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            revision: self.revision,
            version: self.version,
            rotation_days: self.rotation_days,
            amount: self.amount,
            owner_id: self.owner_id.clone(),
            currency: self.currency.clone(),
            tags: self.tags.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a retention milestone in status new.
pub fn new_retention_milestone(code: &str, name: &str) -> RetentionMilestone {
    RetentionMilestone {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        revision: 0,
        version: 0,
        rotation_days: 0,
        amount: 0.0,
        owner_id: "".to_string(),
        currency: "".to_string(),
        tags: Vec::new(),
        quota_units: 0,
        status: RetentionMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_retention_milestone("code-43", "name-793");
        assert_eq!(item.status, RetentionMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_retention_milestone("code-43", "name-793");
        if item.move_to(RetentionMilestoneStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionMilestoneStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_retention_milestone("code-43", "name-793");
        item.status = RetentionMilestoneStatus::Failed;
        assert!(
            item.move_to(RetentionMilestoneStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
