use super::*;

use acme_kit::text;

/// Where a manual tenant claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTenantClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualTenantClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ManualTenantClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual tenant claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantClaim {
    pub code: String,
    pub title: String,
    pub description: String,
    pub external_ref: String,
    pub timezone: String,
    pub grace_days: i64,
    pub status: ManualTenantClaimStatus,
}

impl ManualTenantClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTenantClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTenantClaimStatus) -> bool {
        match self.status {
            ManualTenantClaimStatus::Idle => {
                match next {
                    ManualTenantClaimStatus::Ready | ManualTenantClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTenantClaimStatus::Warming => {
                match next {
                    ManualTenantClaimStatus::Ready | ManualTenantClaimStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTenantClaimStatus::Ready => {
                return next == ManualTenantClaimStatus::Draining || next == ManualTenantClaimStatus::Stopped;
            }
            ManualTenantClaimStatus::Draining => {
                match next {
                    ManualTenantClaimStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualTenantClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual tenant claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual tenant claim", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> ManualTenantClaim {
        ManualTenantClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            external_ref: self.external_ref.clone(),
            timezone: self.timezone.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual tenant claim in status idle.
pub fn new_manual_tenant_claim(code: &str, title: &str) -> ManualTenantClaim {
    ManualTenantClaim {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        external_ref: "".to_string(),
        timezone: "".to_string(),
        grace_days: 0,
        status: ManualTenantClaimStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_tenant_claim("code-709", "title-729");
        assert_eq!(item.status, ManualTenantClaimStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_manual_tenant_claim("code-709", "title-729");
        if item.move_to(ManualTenantClaimStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTenantClaimStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_tenant_claim("code-709", "title-729");
        item.status = ManualTenantClaimStatus::Stopped;
        assert!(
            item.move_to(ManualTenantClaimStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
