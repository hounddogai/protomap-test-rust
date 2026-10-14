use super::*;

use acme_kit::text;

/// Where a seasonal rollback claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRollbackClaimStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalRollbackClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for SeasonalRollbackClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal rollback claim as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackClaim {
    pub code: String,
    pub name: String,
    pub pinned: bool,
    pub expires_at: i64,
    pub contact_email: String,
    pub key_id: String,
    pub ends_at: i64,
    pub ttl_seconds: i64,
    pub updated_at: i64,
    pub country: String,
    pub cluster: String,
    pub status: SeasonalRollbackClaimStatus,
}

impl SeasonalRollbackClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRollbackClaimStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRollbackClaimStatus) -> bool {
        match self.status {
            SeasonalRollbackClaimStatus::Open => {
                return next == SeasonalRollbackClaimStatus::Resolved;
            }
            SeasonalRollbackClaimStatus::InReview => {
                return next == SeasonalRollbackClaimStatus::Archived;
            }
            SeasonalRollbackClaimStatus::Resolved => {
                match next {
                    SeasonalRollbackClaimStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRollbackClaimStatus::Reopened => {
                match next {
                    SeasonalRollbackClaimStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalRollbackClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal rollback claim {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRollbackClaim {
        SeasonalRollbackClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            expires_at: self.expires_at,
            contact_email: self.contact_email.clone(),
            key_id: self.key_id.clone(),
            ends_at: self.ends_at,
            ttl_seconds: self.ttl_seconds,
            updated_at: self.updated_at,
            country: self.country.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a seasonal rollback claim in status open.
pub fn new_seasonal_rollback_claim(code: &str, name: &str) -> SeasonalRollbackClaim {
    SeasonalRollbackClaim {
        code: code.to_string(),
        name: name.to_string(),
        pinned: false,
        expires_at: 0,
        contact_email: "".to_string(),
        key_id: "".to_string(),
        ends_at: 0,
        ttl_seconds: 0,
        updated_at: 0,
        country: "".to_string(),
        cluster: "".to_string(),
        status: SeasonalRollbackClaimStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_rollback_claim("code-262", "name-202");
        assert_eq!(item.status, SeasonalRollbackClaimStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_seasonal_rollback_claim("code-262", "name-202");
        if item.move_to(SeasonalRollbackClaimStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRollbackClaimStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_rollback_claim("code-262", "name-202");
        item.status = SeasonalRollbackClaimStatus::Archived;
        assert!(
            item.move_to(SeasonalRollbackClaimStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
