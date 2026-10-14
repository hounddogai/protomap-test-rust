use super::*;

use acme_kit::text;

/// Where a plan target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PlanTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PlanTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanTarget {
    pub code: String,
    pub label: String,
    pub name: String,
    pub archived: bool,
    pub ratio: f64,
    pub attempts: i64,
    pub batch_size: i64,
    pub status: PlanTargetStatus,
}

impl PlanTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanTargetStatus) -> bool {
        match self.status {
            PlanTargetStatus::Requested => {
                return next == PlanTargetStatus::Granted || next == PlanTargetStatus::Expired;
            }
            PlanTargetStatus::Granted => {
                match next {
                    PlanTargetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanTargetStatus::Revoked => {
                match next {
                    PlanTargetStatus::Expired => {
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

    pub fn move_to(&mut self, next: PlanTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.attempts as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanTarget {
        PlanTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            archived: self.archived,
            ratio: self.ratio,
            attempts: self.attempts,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a plan target in status requested.
pub fn new_plan_target(code: &str, label: &str) -> PlanTarget {
    PlanTarget {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        archived: false,
        ratio: 0.0,
        attempts: 0,
        batch_size: 0,
        status: PlanTargetStatus::Requested,
    }
}
