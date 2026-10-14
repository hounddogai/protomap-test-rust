use super::*;

use acme_kit::text;

/// Where a audit event plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventPlanStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AuditEventPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AuditEventPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventPlan {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub channel: String,
    pub grace_days: i64,
    pub owner_id: String,
    pub floor_value: f64,
    pub retries: i64,
    pub updated_at: i64,
    pub rotation_days: i64,
    pub quota_units: i64,
    pub status: AuditEventPlanStatus,
}

impl AuditEventPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventPlanStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventPlanStatus) -> bool {
        match self.status {
            AuditEventPlanStatus::Created => {
                match next {
                    AuditEventPlanStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventPlanStatus::Validated => {
                match next {
                    AuditEventPlanStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventPlanStatus::Published => {
                return next == AuditEventPlanStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AuditEventPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "audit event plan", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.grace_days as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventPlan {
        AuditEventPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            channel: self.channel.clone(),
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            floor_value: self.floor_value,
            retries: self.retries,
            updated_at: self.updated_at,
            rotation_days: self.rotation_days,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a audit event plan in status created.
pub fn new_audit_event_plan(code: &str, title: &str) -> AuditEventPlan {
    AuditEventPlan {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        channel: "".to_string(),
        grace_days: 0,
        owner_id: "".to_string(),
        floor_value: 0.0,
        retries: 0,
        updated_at: 0,
        rotation_days: 0,
        quota_units: 0,
        status: AuditEventPlanStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_audit_event_plan("code-319", "title-17");
        assert_eq!(item.status, AuditEventPlanStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_audit_event_plan("code-319", "title-17");
        if item.move_to(AuditEventPlanStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventPlanStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_audit_event_plan("code-319", "title-17");
        item.status = AuditEventPlanStatus::Retired;
        assert!(
            item.move_to(AuditEventPlanStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
