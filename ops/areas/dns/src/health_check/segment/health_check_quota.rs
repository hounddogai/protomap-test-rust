use super::*;

use acme_kit::text;

/// Where a health check quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl HealthCheckQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for HealthCheckQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckQuota {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub capacity: i64,
    pub external_ref: String,
    pub scope: String,
    pub rotation_days: i64,
    pub status: HealthCheckQuotaStatus,
}

impl HealthCheckQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckQuotaStatus) -> bool {
        match self.status {
            HealthCheckQuotaStatus::Created => {
                return next == HealthCheckQuotaStatus::Retired;
            }
            HealthCheckQuotaStatus::Validated => {
                return next == HealthCheckQuotaStatus::Published;
            }
            HealthCheckQuotaStatus::Published => {
                return next == HealthCheckQuotaStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "health check quota", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rotation_days as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckQuota {
        HealthCheckQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            capacity: self.capacity,
            external_ref: self.external_ref.clone(),
            scope: self.scope.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a health check quota in status created.
pub fn new_health_check_quota(code: &str, title: &str) -> HealthCheckQuota {
    HealthCheckQuota {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        capacity: 0,
        external_ref: "".to_string(),
        scope: "".to_string(),
        rotation_days: 0,
        status: HealthCheckQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_health_check_quota("code-334", "title-945");
        assert_eq!(item.status, HealthCheckQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_health_check_quota("code-334", "title-945");
        if item.move_to(HealthCheckQuotaStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckQuotaStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_health_check_quota("code-334", "title-945");
        item.status = HealthCheckQuotaStatus::Retired;
        assert!(
            item.move_to(HealthCheckQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
