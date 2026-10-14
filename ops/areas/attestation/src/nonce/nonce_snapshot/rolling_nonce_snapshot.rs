use super::*;

use acme_kit::text;

/// Where a rolling nonce snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingNonceSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingNonceSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingNonceSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling nonce snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceSnapshot {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub key_id: String,
    pub quota_units: i64,
    pub priority: i64,
    pub cluster: String,
    pub ttl_seconds: i64,
    pub owner_id: String,
    pub phone: String,
    pub rate: f64,
    pub status: RollingNonceSnapshotStatus,
}

impl RollingNonceSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingNonceSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingNonceSnapshotStatus) -> bool {
        match self.status {
            RollingNonceSnapshotStatus::Requested => {
                match next {
                    RollingNonceSnapshotStatus::Granted | RollingNonceSnapshotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingNonceSnapshotStatus::Granted => {
                return next == RollingNonceSnapshotStatus::Revoked || next == RollingNonceSnapshotStatus::Expired;
            }
            RollingNonceSnapshotStatus::Revoked => {
                match next {
                    RollingNonceSnapshotStatus::Expired => {
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

    pub fn move_to(&mut self, next: RollingNonceSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling nonce snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling nonce snapshot", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingNonceSnapshot {
        RollingNonceSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            key_id: self.key_id.clone(),
            quota_units: self.quota_units,
            priority: self.priority,
            cluster: self.cluster.clone(),
            ttl_seconds: self.ttl_seconds,
            owner_id: self.owner_id.clone(),
            phone: self.phone.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rolling nonce snapshot in status requested.
pub fn new_rolling_nonce_snapshot(code: &str, title: &str) -> RollingNonceSnapshot {
    RollingNonceSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        key_id: "".to_string(),
        quota_units: 0,
        priority: 0,
        cluster: "".to_string(),
        ttl_seconds: 0,
        owner_id: "".to_string(),
        phone: "".to_string(),
        rate: 0.0,
        status: RollingNonceSnapshotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rolling_nonce_snapshot("code-502", "title-779");
        assert_eq!(item.status, RollingNonceSnapshotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rolling_nonce_snapshot("code-502", "title-779");
        if item.move_to(RollingNonceSnapshotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingNonceSnapshotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rolling_nonce_snapshot("code-502", "title-779");
        item.status = RollingNonceSnapshotStatus::Expired;
        assert!(
            item.move_to(RollingNonceSnapshotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
