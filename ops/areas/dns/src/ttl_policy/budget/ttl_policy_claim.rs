use super::*;

use acme_kit::text;

/// Where a ttl policy claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TtlPolicyClaimStatus {
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

impl std::fmt::Display for TtlPolicyClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyClaim {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub cost: f64,
    pub priority: i64,
    pub batch_size: i64,
    pub status: TtlPolicyClaimStatus,
}

impl TtlPolicyClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyClaimStatus) -> bool {
        match self.status {
            TtlPolicyClaimStatus::Draft => {
                return next == TtlPolicyClaimStatus::Active;
            }
            TtlPolicyClaimStatus::Pending => {
                return next == TtlPolicyClaimStatus::Suspended;
            }
            TtlPolicyClaimStatus::Active => {
                match next {
                    TtlPolicyClaimStatus::Suspended | TtlPolicyClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyClaimStatus::Suspended => {
                match next {
                    TtlPolicyClaimStatus::Closed => {
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

    pub fn move_to(&mut self, next: TtlPolicyClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.priority as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyClaim {
        TtlPolicyClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            cost: self.cost,
            priority: self.priority,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a ttl policy claim in status draft.
pub fn new_ttl_policy_claim(code: &str, name: &str) -> TtlPolicyClaim {
    TtlPolicyClaim {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        cost: 0.0,
        priority: 0,
        batch_size: 0,
        status: TtlPolicyClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_ttl_policy_claim("code-574", "name-760");
        assert_eq!(item.status, TtlPolicyClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_ttl_policy_claim("code-574", "name-760");
        if item.move_to(TtlPolicyClaimStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyClaimStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_ttl_policy_claim("code-574", "name-760");
        item.status = TtlPolicyClaimStatus::Closed;
        assert!(
            item.move_to(TtlPolicyClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
