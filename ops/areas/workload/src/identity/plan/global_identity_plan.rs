use super::*;

use acme_kit::text;

/// Where a global identity plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalIdentityPlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalIdentityPlanStatus {
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

impl std::fmt::Display for GlobalIdentityPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global identity plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityPlan {
    pub code: String,
    pub name: String,
    pub source: String,
    pub unit: String,
    pub currency: String,
    pub score: f64,
    pub status: GlobalIdentityPlanStatus,
}

impl GlobalIdentityPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalIdentityPlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalIdentityPlanStatus) -> bool {
        match self.status {
            GlobalIdentityPlanStatus::Draft => {
                match next {
                    GlobalIdentityPlanStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIdentityPlanStatus::Pending => {
                return next == GlobalIdentityPlanStatus::Active || next == GlobalIdentityPlanStatus::Suspended;
            }
            GlobalIdentityPlanStatus::Active => {
                match next {
                    GlobalIdentityPlanStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIdentityPlanStatus::Suspended => {
                return next == GlobalIdentityPlanStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalIdentityPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global identity plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> GlobalIdentityPlan {
        GlobalIdentityPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            unit: self.unit.clone(),
            currency: self.currency.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a global identity plan in status draft.
pub fn new_global_identity_plan(code: &str, name: &str) -> GlobalIdentityPlan {
    GlobalIdentityPlan {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        unit: "".to_string(),
        currency: "".to_string(),
        score: 0.0,
        status: GlobalIdentityPlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_identity_plan("code-430", "name-656");
        assert_eq!(item.status, GlobalIdentityPlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_global_identity_plan("code-430", "name-656");
        if item.move_to(GlobalIdentityPlanStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalIdentityPlanStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_identity_plan("code-430", "name-656");
        item.status = GlobalIdentityPlanStatus::Closed;
        assert!(
            item.move_to(GlobalIdentityPlanStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
