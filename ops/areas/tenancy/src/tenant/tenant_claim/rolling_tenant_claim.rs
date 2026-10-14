use super::*;

use acme_kit::text;

/// Where a rolling tenant claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingTenantClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingTenantClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RollingTenantClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling tenant claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantClaim {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub rotation_days: i64,
    pub checksum: String,
    pub pinned: bool,
    pub cost: f64,
    pub status: RollingTenantClaimStatus,
}

impl RollingTenantClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingTenantClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingTenantClaimStatus) -> bool {
        match self.status {
            RollingTenantClaimStatus::Planned => {
                return next == RollingTenantClaimStatus::Scheduled || next == RollingTenantClaimStatus::InProgress;
            }
            RollingTenantClaimStatus::Scheduled => {
                match next {
                    RollingTenantClaimStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingTenantClaimStatus::InProgress => {
                return next == RollingTenantClaimStatus::Cancelled;
            }
            RollingTenantClaimStatus::Completed => {
                match next {
                    RollingTenantClaimStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollingTenantClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling tenant claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rotation_days as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingTenantClaim {
        RollingTenantClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            rotation_days: self.rotation_days,
            checksum: self.checksum.clone(),
            pinned: self.pinned,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling tenant claim in status planned.
pub fn new_rolling_tenant_claim(code: &str, name: &str) -> RollingTenantClaim {
    RollingTenantClaim {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        rotation_days: 0,
        checksum: "".to_string(),
        pinned: false,
        cost: 0.0,
        status: RollingTenantClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_tenant_claim("code-982", "name-654");
        assert_eq!(item.status, RollingTenantClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rolling_tenant_claim("code-982", "name-654");
        if item.move_to(RollingTenantClaimStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingTenantClaimStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_tenant_claim("code-982", "name-654");
        item.status = RollingTenantClaimStatus::Cancelled;
        assert!(
            item.move_to(RollingTenantClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
