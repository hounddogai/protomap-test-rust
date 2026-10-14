use super::*;

use acme_kit::text;

/// Where a batched token notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTokenNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedTokenNoticeStatus {
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

impl std::fmt::Display for BatchedTokenNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched token notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenNotice {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub category: String,
    pub sequence: i64,
    pub ttl_seconds: i64,
    pub status: BatchedTokenNoticeStatus,
}

impl BatchedTokenNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTokenNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTokenNoticeStatus) -> bool {
        match self.status {
            BatchedTokenNoticeStatus::Idle => {
                match next {
                    BatchedTokenNoticeStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTokenNoticeStatus::Warming => {
                return next == BatchedTokenNoticeStatus::Draining;
            }
            BatchedTokenNoticeStatus::Ready => {
                return next == BatchedTokenNoticeStatus::Draining || next == BatchedTokenNoticeStatus::Stopped;
            }
            BatchedTokenNoticeStatus::Draining => {
                match next {
                    BatchedTokenNoticeStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedTokenNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched token notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched token notice", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.limit as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedTokenNotice {
        BatchedTokenNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            category: self.category.clone(),
            sequence: self.sequence,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched token notice in status idle.
pub fn new_batched_token_notice(code: &str, name: &str) -> BatchedTokenNotice {
    BatchedTokenNotice {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        category: "".to_string(),
        sequence: 0,
        ttl_seconds: 0,
        status: BatchedTokenNoticeStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_token_notice("code-534", "name-9");
        assert_eq!(item.status, BatchedTokenNoticeStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_batched_token_notice("code-534", "name-9");
        if item.move_to(BatchedTokenNoticeStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTokenNoticeStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_token_notice("code-534", "name-9");
        item.status = BatchedTokenNoticeStatus::Stopped;
        assert!(
            item.move_to(BatchedTokenNoticeStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
