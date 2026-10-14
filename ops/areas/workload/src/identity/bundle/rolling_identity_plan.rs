use super::*;

use acme_kit::text;

/// Where a rolling identity plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingIdentityPlanStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingIdentityPlanStatus {
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

impl std::fmt::Display for RollingIdentityPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling identity plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityPlan {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub principal: String,
    pub revision: i64,
    pub ends_at: i64,
    pub status: RollingIdentityPlanStatus,
}

impl RollingIdentityPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingIdentityPlanStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingIdentityPlanStatus) -> bool {
        match self.status {
            RollingIdentityPlanStatus::Draft => {
                match next {
                    RollingIdentityPlanStatus::Suspended | RollingIdentityPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingIdentityPlanStatus::Pending => {
                return next == RollingIdentityPlanStatus::Closed;
            }
            RollingIdentityPlanStatus::Active => {
                match next {
                    RollingIdentityPlanStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingIdentityPlanStatus::Suspended => {
                return next == RollingIdentityPlanStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingIdentityPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling identity plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.grace_days as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingIdentityPlan {
        RollingIdentityPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            principal: self.principal.clone(),
            revision: self.revision,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling identity plan in status draft.
pub fn new_rolling_identity_plan(code: &str, title: &str) -> RollingIdentityPlan {
    RollingIdentityPlan {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        principal: "".to_string(),
        revision: 0,
        ends_at: 0,
        status: RollingIdentityPlanStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_identity_plan("code-549", "title-834");
        assert_eq!(item.status, RollingIdentityPlanStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_rolling_identity_plan("code-549", "title-834");
        if item.move_to(RollingIdentityPlanStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingIdentityPlanStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_identity_plan("code-549", "title-834");
        item.status = RollingIdentityPlanStatus::Closed;
        assert!(
            item.move_to(RollingIdentityPlanStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
