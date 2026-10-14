use super::*;

use acme_kit::text;

/// Where a global plan review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPlanReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalPlanReviewStatus {
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

impl std::fmt::Display for GlobalPlanReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global plan review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanReview {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub version: i64,
    pub sequence: i64,
    pub duration_minutes: i64,
    pub maximum: f64,
    pub visible: bool,
    pub status: GlobalPlanReviewStatus,
}

impl GlobalPlanReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPlanReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPlanReviewStatus) -> bool {
        match self.status {
            GlobalPlanReviewStatus::New => {
                return next == GlobalPlanReviewStatus::Failed;
            }
            GlobalPlanReviewStatus::Queued => {
                match next {
                    GlobalPlanReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPlanReviewStatus::Running => {
                match next {
                    GlobalPlanReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPlanReviewStatus::Done => {
                return next == GlobalPlanReviewStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalPlanReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global plan review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPlanReview {
        GlobalPlanReview {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            version: self.version,
            sequence: self.sequence,
            duration_minutes: self.duration_minutes,
            maximum: self.maximum,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global plan review in status new.
pub fn new_global_plan_review(code: &str, name: &str) -> GlobalPlanReview {
    GlobalPlanReview {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        version: 0,
        sequence: 0,
        duration_minutes: 0,
        maximum: 0.0,
        visible: false,
        status: GlobalPlanReviewStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_plan_review("code-67", "name-423");
        assert_eq!(item.status, GlobalPlanReviewStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_global_plan_review("code-67", "name-423");
        if item.move_to(GlobalPlanReviewStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPlanReviewStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_plan_review("code-67", "name-423");
        item.status = GlobalPlanReviewStatus::Failed;
        assert!(
            item.move_to(GlobalPlanReviewStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
