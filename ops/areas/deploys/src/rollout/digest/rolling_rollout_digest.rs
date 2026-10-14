use super::*;

use acme_kit::text;

/// Where a rolling rollout digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRolloutDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingRolloutDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingRolloutDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling rollout digest as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRolloutDigest {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub updated_at: i64,
    pub expires_at: i64,
    pub category: String,
    pub contact_email: String,
    pub ttl_seconds: i64,
    pub region: String,
    pub margin: f64,
    pub floor_value: f64,
    pub status: RollingRolloutDigestStatus,
}

impl RollingRolloutDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRolloutDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRolloutDigestStatus) -> bool {
        match self.status {
            RollingRolloutDigestStatus::Requested => {
                match next {
                    RollingRolloutDigestStatus::Revoked | RollingRolloutDigestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRolloutDigestStatus::Granted => {
                match next {
                    RollingRolloutDigestStatus::Revoked | RollingRolloutDigestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRolloutDigestStatus::Revoked => {
                return next == RollingRolloutDigestStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingRolloutDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling rollout digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        (self.margin * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRolloutDigest {
        RollingRolloutDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            updated_at: self.updated_at,
            expires_at: self.expires_at,
            category: self.category.clone(),
            contact_email: self.contact_email.clone(),
            ttl_seconds: self.ttl_seconds,
            region: self.region.clone(),
            margin: self.margin,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rolling rollout digest in status requested.
pub fn new_rolling_rollout_digest(code: &str, name: &str) -> RollingRolloutDigest {
    RollingRolloutDigest {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        updated_at: 0,
        expires_at: 0,
        category: "".to_string(),
        contact_email: "".to_string(),
        ttl_seconds: 0,
        region: "".to_string(),
        margin: 0.0,
        floor_value: 0.0,
        status: RollingRolloutDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_rollout_digest("code-959", "name-846");
        assert_eq!(item.status, RollingRolloutDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rolling_rollout_digest("code-959", "name-846");
        if item.move_to(RollingRolloutDigestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingRolloutDigestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_rollout_digest("code-959", "name-846");
        item.status = RollingRolloutDigestStatus::Expired;
        assert!(
            item.move_to(RollingRolloutDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
