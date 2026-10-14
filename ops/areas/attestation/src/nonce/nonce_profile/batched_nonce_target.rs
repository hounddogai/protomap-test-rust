use super::*;

use acme_kit::text;

/// Where a batched nonce target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedNonceTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedNonceTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedNonceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched nonce target as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceTarget {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub retries: i64,
    pub created_at: i64,
    pub fingerprint: String,
    pub tags: Vec<String>,
    pub limit: i64,
    pub status: BatchedNonceTargetStatus,
}

impl BatchedNonceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedNonceTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedNonceTargetStatus) -> bool {
        match self.status {
            BatchedNonceTargetStatus::Idle => {
                match next {
                    BatchedNonceTargetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedNonceTargetStatus::Warming => {
                return next == BatchedNonceTargetStatus::Draining;
            }
            BatchedNonceTargetStatus::Ready => {
                match next {
                    BatchedNonceTargetStatus::Draining | BatchedNonceTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedNonceTargetStatus::Draining => {
                return next == BatchedNonceTargetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedNonceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched nonce target {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.retries as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedNonceTarget {
        BatchedNonceTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            retries: self.retries,
            created_at: self.created_at,
            fingerprint: self.fingerprint.clone(),
            tags: self.tags.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched nonce target in status idle.
pub fn new_batched_nonce_target(code: &str, label: &str) -> BatchedNonceTarget {
    BatchedNonceTarget {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        retries: 0,
        created_at: 0,
        fingerprint: "".to_string(),
        tags: Vec::new(),
        limit: 0,
        status: BatchedNonceTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_nonce_target("code-553", "label-41");
        assert_eq!(item.status, BatchedNonceTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_batched_nonce_target("code-553", "label-41");
        if item.move_to(BatchedNonceTargetStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedNonceTargetStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_nonce_target("code-553", "label-41");
        item.status = BatchedNonceTargetStatus::Stopped;
        assert!(
            item.move_to(BatchedNonceTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
