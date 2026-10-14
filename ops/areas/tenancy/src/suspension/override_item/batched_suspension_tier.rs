use super::*;

use acme_kit::text;

/// Where a batched suspension tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSuspensionTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedSuspensionTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedSuspensionTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched suspension tier as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionTier {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub ends_at: i64,
    pub approved: bool,
    pub description: String,
    pub batch_size: i64,
    pub notes: String,
    pub issuer: String,
    pub status: BatchedSuspensionTierStatus,
}

impl BatchedSuspensionTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSuspensionTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSuspensionTierStatus) -> bool {
        match self.status {
            BatchedSuspensionTierStatus::Requested => {
                match next {
                    BatchedSuspensionTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSuspensionTierStatus::Granted => {
                return next == BatchedSuspensionTierStatus::Revoked || next == BatchedSuspensionTierStatus::Expired;
            }
            BatchedSuspensionTierStatus::Revoked => {
                return next == BatchedSuspensionTierStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSuspensionTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched suspension tier {} cannot move from {} to {}",
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
        format!("{} {}", "batched suspension tier", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ends_at as f64 + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSuspensionTier {
        BatchedSuspensionTier {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            ends_at: self.ends_at,
            approved: self.approved,
            description: self.description.clone(),
            batch_size: self.batch_size,
            notes: self.notes.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched suspension tier in status requested.
pub fn new_batched_suspension_tier(code: &str, name: &str) -> BatchedSuspensionTier {
    BatchedSuspensionTier {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        ends_at: 0,
        approved: false,
        description: "".to_string(),
        batch_size: 0,
        notes: "".to_string(),
        issuer: "".to_string(),
        status: BatchedSuspensionTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_suspension_tier("code-436", "name-7");
        assert_eq!(item.status, BatchedSuspensionTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_batched_suspension_tier("code-436", "name-7");
        if item.move_to(BatchedSuspensionTierStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSuspensionTierStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_suspension_tier("code-436", "name-7");
        item.status = BatchedSuspensionTierStatus::Expired;
        assert!(
            item.move_to(BatchedSuspensionTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
