use super::*;

use acme_kit::text;

/// Where a policy claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicyClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PolicyClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PolicyClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A policy claim as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyClaim {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub scope: String,
    pub timezone: String,
    pub external_ref: String,
    pub audience: String,
    pub segment: String,
    pub ttl_seconds: i64,
    pub status: PolicyClaimStatus,
}

impl PolicyClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicyClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicyClaimStatus) -> bool {
        match self.status {
            PolicyClaimStatus::Created => {
                match next {
                    PolicyClaimStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicyClaimStatus::Validated => {
                return next == PolicyClaimStatus::Published || next == PolicyClaimStatus::Retired;
            }
            PolicyClaimStatus::Published => {
                return next == PolicyClaimStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicyClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "policy claim", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> PolicyClaim {
        PolicyClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            scope: self.scope.clone(),
            timezone: self.timezone.clone(),
            external_ref: self.external_ref.clone(),
            audience: self.audience.clone(),
            segment: self.segment.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a policy claim in status created.
pub fn new_policy_claim(code: &str, name: &str) -> PolicyClaim {
    PolicyClaim {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        scope: "".to_string(),
        timezone: "".to_string(),
        external_ref: "".to_string(),
        audience: "".to_string(),
        segment: "".to_string(),
        ttl_seconds: 0,
        status: PolicyClaimStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_policy_claim("code-849", "name-665");
        assert_eq!(item.status, PolicyClaimStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_policy_claim("code-849", "name-665");
        if item.move_to(PolicyClaimStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicyClaimStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_policy_claim("code-849", "name-665");
        item.status = PolicyClaimStatus::Retired;
        assert!(
            item.move_to(PolicyClaimStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
