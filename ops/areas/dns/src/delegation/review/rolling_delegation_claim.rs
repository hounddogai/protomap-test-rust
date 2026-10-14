use super::*;

use acme_kit::text;

/// Where a rolling delegation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingDelegationClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingDelegationClaimStatus {
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

impl std::fmt::Display for RollingDelegationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling delegation claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationClaim {
    pub code: String,
    pub title: String,
    pub source: String,
    pub category: String,
    pub key_id: String,
    pub grace_days: i64,
    pub description: String,
    pub created_at: i64,
    pub issuer: String,
    pub phone: String,
    pub threshold: f64,
    pub status: RollingDelegationClaimStatus,
}

impl RollingDelegationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingDelegationClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingDelegationClaimStatus) -> bool {
        match self.status {
            RollingDelegationClaimStatus::Draft => {
                return next == RollingDelegationClaimStatus::Closed;
            }
            RollingDelegationClaimStatus::Pending => {
                return next == RollingDelegationClaimStatus::Active || next == RollingDelegationClaimStatus::Closed;
            }
            RollingDelegationClaimStatus::Active => {
                return next == RollingDelegationClaimStatus::Suspended || next == RollingDelegationClaimStatus::Closed;
            }
            RollingDelegationClaimStatus::Suspended => {
                match next {
                    RollingDelegationClaimStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollingDelegationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling delegation claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.threshold + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingDelegationClaim {
        RollingDelegationClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            category: self.category.clone(),
            key_id: self.key_id.clone(),
            grace_days: self.grace_days,
            description: self.description.clone(),
            created_at: self.created_at,
            issuer: self.issuer.clone(),
            phone: self.phone.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling delegation claim in status draft.
pub fn new_rolling_delegation_claim(code: &str, title: &str) -> RollingDelegationClaim {
    RollingDelegationClaim {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        category: "".to_string(),
        key_id: "".to_string(),
        grace_days: 0,
        description: "".to_string(),
        created_at: 0,
        issuer: "".to_string(),
        phone: "".to_string(),
        threshold: 0.0,
        status: RollingDelegationClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_delegation_claim("code-913", "title-469");
        assert_eq!(item.status, RollingDelegationClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_rolling_delegation_claim("code-913", "title-469");
        if item.move_to(RollingDelegationClaimStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingDelegationClaimStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_delegation_claim("code-913", "title-469");
        item.status = RollingDelegationClaimStatus::Closed;
        assert!(
            item.move_to(RollingDelegationClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
