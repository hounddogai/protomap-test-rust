use super::*;

use acme_kit::text;

/// Where a pending grant plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingGrantPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingGrantPlanStatus {
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

impl std::fmt::Display for PendingGrantPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending grant plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingGrantPlan {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub rotation_days: i64,
    pub region: String,
    pub minimum: f64,
    pub score: f64,
    pub issuer: String,
    pub status: PendingGrantPlanStatus,
}

impl PendingGrantPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingGrantPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingGrantPlanStatus) -> bool {
        match self.status {
            PendingGrantPlanStatus::New => {
                return next == PendingGrantPlanStatus::Running;
            }
            PendingGrantPlanStatus::Queued => {
                return next == PendingGrantPlanStatus::Done || next == PendingGrantPlanStatus::Failed;
            }
            PendingGrantPlanStatus::Running => {
                match next {
                    PendingGrantPlanStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingGrantPlanStatus::Done => {
                return next == PendingGrantPlanStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingGrantPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending grant plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.score * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingGrantPlan {
        PendingGrantPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            rotation_days: self.rotation_days,
            region: self.region.clone(),
            minimum: self.minimum,
            score: self.score,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a pending grant plan in status new.
pub fn new_pending_grant_plan(code: &str, name: &str) -> PendingGrantPlan {
    PendingGrantPlan {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        rotation_days: 0,
        region: "".to_string(),
        minimum: 0.0,
        score: 0.0,
        issuer: "".to_string(),
        status: PendingGrantPlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_grant_plan("code-222", "name-209");
        assert_eq!(item.status, PendingGrantPlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_pending_grant_plan("code-222", "name-209");
        if item.move_to(PendingGrantPlanStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingGrantPlanStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_grant_plan("code-222", "name-209");
        item.status = PendingGrantPlanStatus::Failed;
        assert!(
            item.move_to(PendingGrantPlanStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
