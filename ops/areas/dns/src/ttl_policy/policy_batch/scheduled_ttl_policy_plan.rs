use super::*;

use acme_kit::text;

/// Where a scheduled ttl policy plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTtlPolicyPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledTtlPolicyPlanStatus {
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

impl std::fmt::Display for ScheduledTtlPolicyPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled ttl policy plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyPlan {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub host: String,
    pub quantity: i64,
    pub summary: String,
    pub sequence: i64,
    pub expires_at: i64,
    pub quota_units: i64,
    pub key_id: String,
    pub principal: String,
    pub status: ScheduledTtlPolicyPlanStatus,
}

impl ScheduledTtlPolicyPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTtlPolicyPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTtlPolicyPlanStatus) -> bool {
        match self.status {
            ScheduledTtlPolicyPlanStatus::New => {
                return next == ScheduledTtlPolicyPlanStatus::Queued || next == ScheduledTtlPolicyPlanStatus::Done;
            }
            ScheduledTtlPolicyPlanStatus::Queued => {
                match next {
                    ScheduledTtlPolicyPlanStatus::Running | ScheduledTtlPolicyPlanStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTtlPolicyPlanStatus::Running => {
                match next {
                    ScheduledTtlPolicyPlanStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTtlPolicyPlanStatus::Done => {
                match next {
                    ScheduledTtlPolicyPlanStatus::Failed => {
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

    pub fn move_to(&mut self, next: ScheduledTtlPolicyPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled ttl policy plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled ttl policy plan", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quota_units as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTtlPolicyPlan {
        ScheduledTtlPolicyPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            host: self.host.clone(),
            quantity: self.quantity,
            summary: self.summary.clone(),
            sequence: self.sequence,
            expires_at: self.expires_at,
            quota_units: self.quota_units,
            key_id: self.key_id.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled ttl policy plan in status new.
pub fn new_scheduled_ttl_policy_plan(code: &str, title: &str) -> ScheduledTtlPolicyPlan {
    ScheduledTtlPolicyPlan {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        host: "".to_string(),
        quantity: 0,
        summary: "".to_string(),
        sequence: 0,
        expires_at: 0,
        quota_units: 0,
        key_id: "".to_string(),
        principal: "".to_string(),
        status: ScheduledTtlPolicyPlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_ttl_policy_plan("code-52", "title-69");
        assert_eq!(item.status, ScheduledTtlPolicyPlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_scheduled_ttl_policy_plan("code-52", "title-69");
        if item.move_to(ScheduledTtlPolicyPlanStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTtlPolicyPlanStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_ttl_policy_plan("code-52", "title-69");
        item.status = ScheduledTtlPolicyPlanStatus::Failed;
        assert!(
            item.move_to(ScheduledTtlPolicyPlanStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
