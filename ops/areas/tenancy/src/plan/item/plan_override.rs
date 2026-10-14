use super::*;

use acme_kit::text;

/// Where a plan override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanOverrideStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PlanOverrideStatus {
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

impl std::fmt::Display for PlanOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan override as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanOverride {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub checksum: String,
    pub batch_size: i64,
    pub budget: f64,
    pub priority: i64,
    pub revision: i64,
    pub notes: String,
    pub status: PlanOverrideStatus,
}

impl PlanOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanOverrideStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanOverrideStatus) -> bool {
        match self.status {
            PlanOverrideStatus::New => {
                return next == PlanOverrideStatus::Done || next == PlanOverrideStatus::Failed;
            }
            PlanOverrideStatus::Queued => {
                match next {
                    PlanOverrideStatus::Done | PlanOverrideStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanOverrideStatus::Running => {
                return next == PlanOverrideStatus::Failed;
            }
            PlanOverrideStatus::Done => {
                return next == PlanOverrideStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "plan override", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.revision as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanOverride {
        PlanOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            checksum: self.checksum.clone(),
            batch_size: self.batch_size,
            budget: self.budget,
            priority: self.priority,
            revision: self.revision,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a plan override in status new.
pub fn new_plan_override(code: &str, label: &str) -> PlanOverride {
    PlanOverride {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        checksum: "".to_string(),
        batch_size: 0,
        budget: 0.0,
        priority: 0,
        revision: 0,
        notes: "".to_string(),
        status: PlanOverrideStatus::New,
    }
}
