use super::*;

use acme_kit::text;

/// Where a batched silence target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSilenceTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedSilenceTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedSilenceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched silence target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSilenceTarget {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub capacity: i64,
    pub ttl_seconds: i64,
    pub active: bool,
    pub ratio: f64,
    pub pinned: bool,
    pub status: BatchedSilenceTargetStatus,
}

impl BatchedSilenceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSilenceTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSilenceTargetStatus) -> bool {
        match self.status {
            BatchedSilenceTargetStatus::Requested => {
                match next {
                    BatchedSilenceTargetStatus::Granted | BatchedSilenceTargetStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSilenceTargetStatus::Granted => {
                return next == BatchedSilenceTargetStatus::Revoked;
            }
            BatchedSilenceTargetStatus::Revoked => {
                match next {
                    BatchedSilenceTargetStatus::Expired => {
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

    pub fn move_to(&mut self, next: BatchedSilenceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched silence target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSilenceTarget {
        BatchedSilenceTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            capacity: self.capacity,
            ttl_seconds: self.ttl_seconds,
            active: self.active,
            ratio: self.ratio,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched silence target in status requested.
pub fn new_batched_silence_target(code: &str, name: &str) -> BatchedSilenceTarget {
    BatchedSilenceTarget {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        capacity: 0,
        ttl_seconds: 0,
        active: false,
        ratio: 0.0,
        pinned: false,
        status: BatchedSilenceTargetStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_silence_target("code-340", "name-179");
        assert_eq!(item.status, BatchedSilenceTargetStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_batched_silence_target("code-340", "name-179");
        if item.move_to(BatchedSilenceTargetStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSilenceTargetStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_silence_target("code-340", "name-179");
        item.status = BatchedSilenceTargetStatus::Expired;
        assert!(
            item.move_to(BatchedSilenceTargetStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
