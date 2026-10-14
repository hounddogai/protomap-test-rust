use super::*;

use acme_kit::text;

/// Where a ttl policy checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyCheckpointStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TtlPolicyCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TtlPolicyCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy checkpoint as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyCheckpoint {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub ends_at: i64,
    pub key_id: String,
    pub budget: f64,
    pub approved: bool,
    pub quantity: i64,
    pub active: bool,
    pub issuer: String,
    pub status: TtlPolicyCheckpointStatus,
}

impl TtlPolicyCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyCheckpointStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyCheckpointStatus) -> bool {
        match self.status {
            TtlPolicyCheckpointStatus::Requested => {
                return next == TtlPolicyCheckpointStatus::Expired;
            }
            TtlPolicyCheckpointStatus::Granted => {
                match next {
                    TtlPolicyCheckpointStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyCheckpointStatus::Revoked => {
                return next == TtlPolicyCheckpointStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.port as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyCheckpoint {
        TtlPolicyCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            ends_at: self.ends_at,
            key_id: self.key_id.clone(),
            budget: self.budget,
            approved: self.approved,
            quantity: self.quantity,
            active: self.active,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a ttl policy checkpoint in status requested.
pub fn new_ttl_policy_checkpoint(code: &str, name: &str) -> TtlPolicyCheckpoint {
    TtlPolicyCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        ends_at: 0,
        key_id: "".to_string(),
        budget: 0.0,
        approved: false,
        quantity: 0,
        active: false,
        issuer: "".to_string(),
        status: TtlPolicyCheckpointStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_ttl_policy_checkpoint("code-540", "name-412");
        assert_eq!(item.status, TtlPolicyCheckpointStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_ttl_policy_checkpoint("code-540", "name-412");
        if item.move_to(TtlPolicyCheckpointStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyCheckpointStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_ttl_policy_checkpoint("code-540", "name-412");
        item.status = TtlPolicyCheckpointStatus::Expired;
        assert!(
            item.move_to(TtlPolicyCheckpointStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
