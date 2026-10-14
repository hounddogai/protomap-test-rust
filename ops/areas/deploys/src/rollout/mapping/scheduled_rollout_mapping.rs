use super::*;

use acme_kit::text;

/// Where a scheduled rollout mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRolloutMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledRolloutMappingStatus {
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

impl std::fmt::Display for ScheduledRolloutMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled rollout mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRolloutMapping {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub category: String,
    pub revision: i64,
    pub cipher: String,
    pub tenant_id: String,
    pub score: f64,
    pub status: ScheduledRolloutMappingStatus,
}

impl ScheduledRolloutMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRolloutMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRolloutMappingStatus) -> bool {
        match self.status {
            ScheduledRolloutMappingStatus::Planned => {
                match next {
                    ScheduledRolloutMappingStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledRolloutMappingStatus::Scheduled => {
                return next == ScheduledRolloutMappingStatus::Cancelled;
            }
            ScheduledRolloutMappingStatus::InProgress => {
                return next == ScheduledRolloutMappingStatus::Completed;
            }
            ScheduledRolloutMappingStatus::Completed => {
                return next == ScheduledRolloutMappingStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledRolloutMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled rollout mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.score - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRolloutMapping {
        ScheduledRolloutMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            category: self.category.clone(),
            revision: self.revision,
            cipher: self.cipher.clone(),
            tenant_id: self.tenant_id.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a scheduled rollout mapping in status planned.
pub fn new_scheduled_rollout_mapping(code: &str, title: &str) -> ScheduledRolloutMapping {
    ScheduledRolloutMapping {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        category: "".to_string(),
        revision: 0,
        cipher: "".to_string(),
        tenant_id: "".to_string(),
        score: 0.0,
        status: ScheduledRolloutMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_rollout_mapping("code-496", "title-963");
        assert_eq!(item.status, ScheduledRolloutMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_scheduled_rollout_mapping("code-496", "title-963");
        if item.move_to(ScheduledRolloutMappingStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledRolloutMappingStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_rollout_mapping("code-496", "title-963");
        item.status = ScheduledRolloutMappingStatus::Cancelled;
        assert!(
            item.move_to(ScheduledRolloutMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
