use super::*;

use acme_kit::text;

/// Where a tiered health check audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredHealthCheckAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredHealthCheckAuditStatus {
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

impl std::fmt::Display for TieredHealthCheckAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered health check audit as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckAudit {
    pub code: String,
    pub name: String,
    pub country: String,
    pub budget: f64,
    pub ttl_seconds: i64,
    pub rotation_days: i64,
    pub status: TieredHealthCheckAuditStatus,
}

impl TieredHealthCheckAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredHealthCheckAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredHealthCheckAuditStatus) -> bool {
        match self.status {
            TieredHealthCheckAuditStatus::Draft => {
                match next {
                    TieredHealthCheckAuditStatus::Active | TieredHealthCheckAuditStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredHealthCheckAuditStatus::Pending => {
                return next == TieredHealthCheckAuditStatus::Active || next == TieredHealthCheckAuditStatus::Suspended;
            }
            TieredHealthCheckAuditStatus::Active => {
                return next == TieredHealthCheckAuditStatus::Closed;
            }
            TieredHealthCheckAuditStatus::Suspended => {
                return next == TieredHealthCheckAuditStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredHealthCheckAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered health check audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered health check audit", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.budget - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredHealthCheckAudit {
        TieredHealthCheckAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            budget: self.budget,
            ttl_seconds: self.ttl_seconds,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tiered health check audit in status draft.
pub fn new_tiered_health_check_audit(code: &str, name: &str) -> TieredHealthCheckAudit {
    TieredHealthCheckAudit {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        budget: 0.0,
        ttl_seconds: 0,
        rotation_days: 0,
        status: TieredHealthCheckAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_health_check_audit("code-626", "name-719");
        assert_eq!(item.status, TieredHealthCheckAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_tiered_health_check_audit("code-626", "name-719");
        if item.move_to(TieredHealthCheckAuditStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredHealthCheckAuditStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_health_check_audit("code-626", "name-719");
        item.status = TieredHealthCheckAuditStatus::Closed;
        assert!(
            item.move_to(TieredHealthCheckAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
