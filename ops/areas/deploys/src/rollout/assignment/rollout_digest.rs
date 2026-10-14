use super::*;

use acme_kit::text;

/// Where a rollout digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutDigestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RolloutDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RolloutDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollout digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutDigest {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub cipher: String,
    pub starts_at: i64,
    pub label: String,
    pub attempts: i64,
    pub status: RolloutDigestStatus,
}

impl RolloutDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutDigestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutDigestStatus) -> bool {
        match self.status {
            RolloutDigestStatus::Requested => {
                return next == RolloutDigestStatus::Revoked;
            }
            RolloutDigestStatus::Granted => {
                match next {
                    RolloutDigestStatus::Revoked | RolloutDigestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutDigestStatus::Revoked => {
                match next {
                    RolloutDigestStatus::Expired => {
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

    pub fn move_to(&mut self, next: RolloutDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.quantity as f64) * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutDigest {
        RolloutDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            cipher: self.cipher.clone(),
            starts_at: self.starts_at,
            label: self.label.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollout digest in status requested.
pub fn new_rollout_digest(code: &str, name: &str) -> RolloutDigest {
    RolloutDigest {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        cipher: "".to_string(),
        starts_at: 0,
        label: "".to_string(),
        attempts: 0,
        status: RolloutDigestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollout_digest("code-801", "name-818");
        assert_eq!(item.status, RolloutDigestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rollout_digest("code-801", "name-818");
        if item.move_to(RolloutDigestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutDigestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollout_digest("code-801", "name-818");
        item.status = RolloutDigestStatus::Expired;
        assert!(
            item.move_to(RolloutDigestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
