use super::*;

use acme_kit::text;

/// Where a global isolation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalIsolationClaimStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalIsolationClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalIsolationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global isolation claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationClaim {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub attempts: i64,
    pub issuer: String,
    pub created_at: i64,
    pub threshold: f64,
    pub starts_at: i64,
    pub quota_units: i64,
    pub status: GlobalIsolationClaimStatus,
}

impl GlobalIsolationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalIsolationClaimStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalIsolationClaimStatus) -> bool {
        match self.status {
            GlobalIsolationClaimStatus::Requested => {
                match next {
                    GlobalIsolationClaimStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIsolationClaimStatus::Granted => {
                return next == GlobalIsolationClaimStatus::Expired;
            }
            GlobalIsolationClaimStatus::Revoked => {
                match next {
                    GlobalIsolationClaimStatus::Expired => {
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

    pub fn move_to(&mut self, next: GlobalIsolationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global isolation claim {} cannot move from {} to {}",
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
        format!("{} {}", "global isolation claim", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.attempts as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalIsolationClaim {
        GlobalIsolationClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            attempts: self.attempts,
            issuer: self.issuer.clone(),
            created_at: self.created_at,
            threshold: self.threshold,
            starts_at: self.starts_at,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global isolation claim in status requested.
pub fn new_global_isolation_claim(code: &str, name: &str) -> GlobalIsolationClaim {
    GlobalIsolationClaim {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        attempts: 0,
        issuer: "".to_string(),
        created_at: 0,
        threshold: 0.0,
        starts_at: 0,
        quota_units: 0,
        status: GlobalIsolationClaimStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_isolation_claim("code-220", "name-729");
        assert_eq!(item.status, GlobalIsolationClaimStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_global_isolation_claim("code-220", "name-729");
        if item.move_to(GlobalIsolationClaimStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalIsolationClaimStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_isolation_claim("code-220", "name-729");
        item.status = GlobalIsolationClaimStatus::Expired;
        assert!(
            item.move_to(GlobalIsolationClaimStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
