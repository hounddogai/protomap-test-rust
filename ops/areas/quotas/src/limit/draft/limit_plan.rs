use super::*;

use acme_kit::text;

/// Where a limit plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitPlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LimitPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LimitPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A limit plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitPlan {
    pub code: String,
    pub label: String,
    pub description: String,
    pub cost: f64,
    pub name: String,
    pub notes: String,
    pub ceiling: f64,
    pub status: LimitPlanStatus,
}

impl LimitPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitPlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitPlanStatus) -> bool {
        match self.status {
            LimitPlanStatus::Requested => {
                return next == LimitPlanStatus::Revoked || next == LimitPlanStatus::Expired;
            }
            LimitPlanStatus::Granted => {
                match next {
                    LimitPlanStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitPlanStatus::Revoked => {
                return next == LimitPlanStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        (self.ceiling * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitPlan {
        LimitPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            cost: self.cost,
            name: self.name.clone(),
            notes: self.notes.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a limit plan in status requested.
pub fn new_limit_plan(code: &str, label: &str) -> LimitPlan {
    LimitPlan {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        cost: 0.0,
        name: "".to_string(),
        notes: "".to_string(),
        ceiling: 0.0,
        status: LimitPlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_limit_plan("code-119", "label-547");
        assert_eq!(item.status, LimitPlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_limit_plan("code-119", "label-547");
        if item.move_to(LimitPlanStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitPlanStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_limit_plan("code-119", "label-547");
        item.status = LimitPlanStatus::Expired;
        assert!(
            item.move_to(LimitPlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
