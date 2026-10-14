use super::*;

use acme_kit::text;

/// Where a rollout plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutPlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RolloutPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RolloutPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout plan as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPlan {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub grace_days: i64,
    pub source: String,
    pub ceiling: f64,
    pub status: RolloutPlanStatus,
}

impl RolloutPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutPlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutPlanStatus) -> bool {
        match self.status {
            RolloutPlanStatus::Draft => {
                match next {
                    RolloutPlanStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutPlanStatus::Pending => {
                match next {
                    RolloutPlanStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutPlanStatus::Active => {
                return next == RolloutPlanStatus::Suspended;
            }
            RolloutPlanStatus::Suspended => {
                return next == RolloutPlanStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rollout plan", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.grace_days as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutPlan {
        RolloutPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            grace_days: self.grace_days,
            source: self.source.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rollout plan in status draft.
pub fn new_rollout_plan(code: &str, title: &str) -> RolloutPlan {
    RolloutPlan {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        grace_days: 0,
        source: "".to_string(),
        ceiling: 0.0,
        status: RolloutPlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rollout_plan("code-73", "title-900");
        assert_eq!(item.status, RolloutPlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_rollout_plan("code-73", "title-900");
        if item.move_to(RolloutPlanStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutPlanStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rollout_plan("code-73", "title-900");
        item.status = RolloutPlanStatus::Closed;
        assert!(item.move_to(RolloutPlanStatus::Draft).is_err(), "expected move to fail");
    }
}
