use super::*;

use acme_kit::text;

/// Where a plan claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PlanClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PlanClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanClaim {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub expires_at: i64,
    pub region: String,
    pub updated_at: i64,
    pub status: PlanClaimStatus,
}

impl PlanClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanClaimStatus) -> bool {
        match self.status {
            PlanClaimStatus::Created => {
                match next {
                    PlanClaimStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanClaimStatus::Validated => {
                return next == PlanClaimStatus::Retired;
            }
            PlanClaimStatus::Published => {
                match next {
                    PlanClaimStatus::Retired => {
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

    pub fn move_to(&mut self, next: PlanClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan claim {} cannot move from {} to {}",
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
        format!("{} {}", "plan claim", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.expires_at as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanClaim {
        PlanClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            expires_at: self.expires_at,
            region: self.region.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a plan claim in status created.
pub fn new_plan_claim(code: &str, name: &str) -> PlanClaim {
    PlanClaim {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        expires_at: 0,
        region: "".to_string(),
        updated_at: 0,
        status: PlanClaimStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_plan_claim("code-45", "name-109");
        assert_eq!(item.status, PlanClaimStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_plan_claim("code-45", "name-109");
        if item.move_to(PlanClaimStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanClaimStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_plan_claim("code-45", "name-109");
        item.status = PlanClaimStatus::Retired;
        assert!(item.move_to(PlanClaimStatus::Created).is_err(), "expected move to fail");
    }
}
