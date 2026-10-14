use super::*;

use acme_kit::text;

/// Where a batched limit claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedLimitClaimStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedLimitClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedLimitClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched limit claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitClaim {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub maximum: f64,
    pub port: i64,
    pub status: BatchedLimitClaimStatus,
}

impl BatchedLimitClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedLimitClaimStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedLimitClaimStatus) -> bool {
        match self.status {
            BatchedLimitClaimStatus::Requested => {
                match next {
                    BatchedLimitClaimStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedLimitClaimStatus::Granted => {
                return next == BatchedLimitClaimStatus::Revoked;
            }
            BatchedLimitClaimStatus::Revoked => {
                match next {
                    BatchedLimitClaimStatus::Expired => {
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

    pub fn move_to(&mut self, next: BatchedLimitClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched limit claim {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched limit claim", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.maximum - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedLimitClaim {
        BatchedLimitClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            maximum: self.maximum,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched limit claim in status requested.
pub fn new_batched_limit_claim(code: &str, label: &str) -> BatchedLimitClaim {
    BatchedLimitClaim {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        maximum: 0.0,
        port: 0,
        status: BatchedLimitClaimStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_limit_claim("code-700", "label-598");
        assert_eq!(item.status, BatchedLimitClaimStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_batched_limit_claim("code-700", "label-598");
        if item.move_to(BatchedLimitClaimStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedLimitClaimStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_limit_claim("code-700", "label-598");
        item.status = BatchedLimitClaimStatus::Expired;
        assert!(
            item.move_to(BatchedLimitClaimStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
