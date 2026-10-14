use super::*;

use acme_kit::text;

/// Where a health check adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckAdjustmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl HealthCheckAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for HealthCheckAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check adjustment as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAdjustment {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub duration_minutes: i64,
    pub expires_at: i64,
    pub created_at: i64,
    pub grace_days: i64,
    pub weight: f64,
    pub cost: f64,
    pub tenant_id: String,
    pub status: HealthCheckAdjustmentStatus,
}

impl HealthCheckAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckAdjustmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckAdjustmentStatus) -> bool {
        match self.status {
            HealthCheckAdjustmentStatus::Proposed => {
                match next {
                    HealthCheckAdjustmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckAdjustmentStatus::Approved => {
                return next == HealthCheckAdjustmentStatus::Rejected;
            }
            HealthCheckAdjustmentStatus::Rejected => {
                return next == HealthCheckAdjustmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check adjustment {} cannot move from {} to {}",
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
        format!("{} {}", "health check adjustment", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.grace_days as f64 + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckAdjustment {
        HealthCheckAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            duration_minutes: self.duration_minutes,
            expires_at: self.expires_at,
            created_at: self.created_at,
            grace_days: self.grace_days,
            weight: self.weight,
            cost: self.cost,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a health check adjustment in status proposed.
pub fn new_health_check_adjustment(code: &str, title: &str) -> HealthCheckAdjustment {
    HealthCheckAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        duration_minutes: 0,
        expires_at: 0,
        created_at: 0,
        grace_days: 0,
        weight: 0.0,
        cost: 0.0,
        tenant_id: "".to_string(),
        status: HealthCheckAdjustmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_health_check_adjustment("code-835", "title-952");
        assert_eq!(item.status, HealthCheckAdjustmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_health_check_adjustment("code-835", "title-952");
        if item.move_to(HealthCheckAdjustmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckAdjustmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_health_check_adjustment("code-835", "title-952");
        item.status = HealthCheckAdjustmentStatus::Withdrawn;
        assert!(
            item.move_to(HealthCheckAdjustmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
