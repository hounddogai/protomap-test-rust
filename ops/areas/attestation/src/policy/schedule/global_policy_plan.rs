use super::*;

use acme_kit::text;

/// Where a global policy plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPolicyPlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalPolicyPlanStatus {
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

impl std::fmt::Display for GlobalPolicyPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global policy plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPolicyPlan {
    pub code: String,
    pub name: String,
    pub score: f64,
    pub cluster: String,
    pub ttl_seconds: i64,
    pub port: i64,
    pub floor_value: f64,
    pub limit: i64,
    pub status: GlobalPolicyPlanStatus,
}

impl GlobalPolicyPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPolicyPlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPolicyPlanStatus) -> bool {
        match self.status {
            GlobalPolicyPlanStatus::Draft => {
                match next {
                    GlobalPolicyPlanStatus::Suspended | GlobalPolicyPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPolicyPlanStatus::Pending => {
                return next == GlobalPolicyPlanStatus::Closed;
            }
            GlobalPolicyPlanStatus::Active => {
                return next == GlobalPolicyPlanStatus::Closed;
            }
            GlobalPolicyPlanStatus::Suspended => {
                match next {
                    GlobalPolicyPlanStatus::Closed => {
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

    pub fn move_to(&mut self, next: GlobalPolicyPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global policy plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.limit as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPolicyPlan {
        GlobalPolicyPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            score: self.score,
            cluster: self.cluster.clone(),
            ttl_seconds: self.ttl_seconds,
            port: self.port,
            floor_value: self.floor_value,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a global policy plan in status draft.
pub fn new_global_policy_plan(code: &str, name: &str) -> GlobalPolicyPlan {
    GlobalPolicyPlan {
        code: code.to_string(),
        name: name.to_string(),
        score: 0.0,
        cluster: "".to_string(),
        ttl_seconds: 0,
        port: 0,
        floor_value: 0.0,
        limit: 0,
        status: GlobalPolicyPlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_global_policy_plan("code-878", "name-442");
        assert_eq!(item.status, GlobalPolicyPlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_global_policy_plan("code-878", "name-442");
        if item.move_to(GlobalPolicyPlanStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalPolicyPlanStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_global_policy_plan("code-878", "name-442");
        item.status = GlobalPolicyPlanStatus::Closed;
        assert!(
            item.move_to(GlobalPolicyPlanStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
