use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched retention plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedRetentionPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedRetentionPlanStatus {
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

impl std::fmt::Display for BatchedRetentionPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched retention plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRetentionPlan {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub duration_minutes: i64,
    pub limit: i64,
    pub name: String,
    pub score: f64,
    pub margin: f64,
    pub discount: f64,
    pub status: BatchedRetentionPlanStatus,
}

impl BatchedRetentionPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedRetentionPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedRetentionPlanStatus) -> bool {
        match self.status {
            BatchedRetentionPlanStatus::New => {
                return next == BatchedRetentionPlanStatus::Done || next == BatchedRetentionPlanStatus::Failed;
            }
            BatchedRetentionPlanStatus::Queued => {
                match next {
                    BatchedRetentionPlanStatus::Running | BatchedRetentionPlanStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedRetentionPlanStatus::Running => {
                return next == BatchedRetentionPlanStatus::Done || next == BatchedRetentionPlanStatus::Failed;
            }
            BatchedRetentionPlanStatus::Done => {
                match next {
                    BatchedRetentionPlanStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedRetentionPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched retention plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.grace_days as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedRetentionPlan {
        BatchedRetentionPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            attributes: self.attributes.clone(),
            duration_minutes: self.duration_minutes,
            limit: self.limit,
            name: self.name.clone(),
            score: self.score,
            margin: self.margin,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched retention plan in status new.
pub fn new_batched_retention_plan(code: &str, title: &str) -> BatchedRetentionPlan {
    BatchedRetentionPlan {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        attributes: BTreeMap::new(),
        duration_minutes: 0,
        limit: 0,
        name: "".to_string(),
        score: 0.0,
        margin: 0.0,
        discount: 0.0,
        status: BatchedRetentionPlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_retention_plan("code-155", "title-241");
        assert_eq!(item.status, BatchedRetentionPlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_batched_retention_plan("code-155", "title-241");
        if item.move_to(BatchedRetentionPlanStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedRetentionPlanStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_retention_plan("code-155", "title-241");
        item.status = BatchedRetentionPlanStatus::Failed;
        assert!(
            item.move_to(BatchedRetentionPlanStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
