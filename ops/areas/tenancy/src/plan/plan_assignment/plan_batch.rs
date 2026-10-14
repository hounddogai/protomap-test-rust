use super::*;

use acme_kit::text;

/// Where a plan batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PlanBatchStatus {
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

impl std::fmt::Display for PlanBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan batch as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanBatch {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub capacity: i64,
    pub starts_at: i64,
    pub pinned: bool,
    pub status: PlanBatchStatus,
}

impl PlanBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanBatchStatus) -> bool {
        match self.status {
            PlanBatchStatus::New => {
                match next {
                    PlanBatchStatus::Queued | PlanBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanBatchStatus::Queued => {
                match next {
                    PlanBatchStatus::Done | PlanBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanBatchStatus::Running => {
                return next == PlanBatchStatus::Done;
            }
            PlanBatchStatus::Done => {
                return next == PlanBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan batch", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.version as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanBatch {
        PlanBatch {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            capacity: self.capacity,
            starts_at: self.starts_at,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a plan batch in status new.
pub fn new_plan_batch(code: &str, title: &str) -> PlanBatch {
    PlanBatch {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        capacity: 0,
        starts_at: 0,
        pinned: false,
        status: PlanBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_plan_batch("code-198", "title-776");
        assert_eq!(item.status, PlanBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_plan_batch("code-198", "title-776");
        if item.move_to(PlanBatchStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanBatchStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_plan_batch("code-198", "title-776");
        item.status = PlanBatchStatus::Failed;
        assert!(item.move_to(PlanBatchStatus::New).is_err(), "expected move to fail");
    }
}
