use super::*;

use acme_kit::text;

/// Where a batched allowlist quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAllowlistQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedAllowlistQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedAllowlistQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched allowlist quota as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistQuota {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub port: i64,
    pub checksum: String,
    pub owner_id: String,
    pub budget: f64,
    pub status: BatchedAllowlistQuotaStatus,
}

impl BatchedAllowlistQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAllowlistQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAllowlistQuotaStatus) -> bool {
        match self.status {
            BatchedAllowlistQuotaStatus::Requested => {
                return next == BatchedAllowlistQuotaStatus::Granted || next == BatchedAllowlistQuotaStatus::Expired;
            }
            BatchedAllowlistQuotaStatus::Granted => {
                return next == BatchedAllowlistQuotaStatus::Revoked;
            }
            BatchedAllowlistQuotaStatus::Revoked => {
                return next == BatchedAllowlistQuotaStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedAllowlistQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched allowlist quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.port as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAllowlistQuota {
        BatchedAllowlistQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            port: self.port,
            checksum: self.checksum.clone(),
            owner_id: self.owner_id.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a batched allowlist quota in status requested.
pub fn new_batched_allowlist_quota(code: &str, name: &str) -> BatchedAllowlistQuota {
    BatchedAllowlistQuota {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        port: 0,
        checksum: "".to_string(),
        owner_id: "".to_string(),
        budget: 0.0,
        status: BatchedAllowlistQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_allowlist_quota("code-535", "name-593");
        assert_eq!(item.status, BatchedAllowlistQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_batched_allowlist_quota("code-535", "name-593");
        if item.move_to(BatchedAllowlistQuotaStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedAllowlistQuotaStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_allowlist_quota("code-535", "name-593");
        item.status = BatchedAllowlistQuotaStatus::Expired;
        assert!(
            item.move_to(BatchedAllowlistQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
