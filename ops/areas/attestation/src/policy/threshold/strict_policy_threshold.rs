use super::*;

use acme_kit::text;

/// Where a strict policy threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictPolicyThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl StrictPolicyThresholdStatus {
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

impl std::fmt::Display for StrictPolicyThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict policy threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyThreshold {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub amount: f64,
    pub limit: i64,
    pub duration_minutes: i64,
    pub threshold: f64,
    pub cost: f64,
    pub archived: bool,
    pub status: StrictPolicyThresholdStatus,
}

impl StrictPolicyThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictPolicyThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictPolicyThresholdStatus) -> bool {
        match self.status {
            StrictPolicyThresholdStatus::New => {
                return next == StrictPolicyThresholdStatus::Running || next == StrictPolicyThresholdStatus::Failed;
            }
            StrictPolicyThresholdStatus::Queued => {
                return next == StrictPolicyThresholdStatus::Running;
            }
            StrictPolicyThresholdStatus::Running => {
                match next {
                    StrictPolicyThresholdStatus::Done | StrictPolicyThresholdStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictPolicyThresholdStatus::Done => {
                return next == StrictPolicyThresholdStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictPolicyThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict policy threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict policy threshold", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.amount + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictPolicyThreshold {
        StrictPolicyThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            amount: self.amount,
            limit: self.limit,
            duration_minutes: self.duration_minutes,
            threshold: self.threshold,
            cost: self.cost,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict policy threshold in status new.
pub fn new_strict_policy_threshold(code: &str, name: &str) -> StrictPolicyThreshold {
    StrictPolicyThreshold {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        amount: 0.0,
        limit: 0,
        duration_minutes: 0,
        threshold: 0.0,
        cost: 0.0,
        archived: false,
        status: StrictPolicyThresholdStatus::New,
    }
}
