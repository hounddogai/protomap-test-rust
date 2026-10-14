use super::*;

use acme_kit::text;

/// Where a batched query tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedQueryTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedQueryTierStatus {
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

impl std::fmt::Display for BatchedQueryTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched query tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQueryTier {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub revision: i64,
    pub host: String,
    pub phone: String,
    pub rate: f64,
    pub burst: i64,
    pub category: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub status: BatchedQueryTierStatus,
}

impl BatchedQueryTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedQueryTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedQueryTierStatus) -> bool {
        match self.status {
            BatchedQueryTierStatus::Idle => {
                match next {
                    BatchedQueryTierStatus::Ready | BatchedQueryTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQueryTierStatus::Warming => {
                match next {
                    BatchedQueryTierStatus::Draining | BatchedQueryTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQueryTierStatus::Ready => {
                match next {
                    BatchedQueryTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQueryTierStatus::Draining => {
                match next {
                    BatchedQueryTierStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedQueryTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched query tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.limit as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedQueryTier {
        BatchedQueryTier {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            revision: self.revision,
            host: self.host.clone(),
            phone: self.phone.clone(),
            rate: self.rate,
            burst: self.burst,
            category: self.category.clone(),
            notes: self.notes.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched query tier in status idle.
pub fn new_batched_query_tier(code: &str, label: &str) -> BatchedQueryTier {
    BatchedQueryTier {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        revision: 0,
        host: "".to_string(),
        phone: "".to_string(),
        rate: 0.0,
        burst: 0,
        category: "".to_string(),
        notes: "".to_string(),
        tags: Vec::new(),
        status: BatchedQueryTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_query_tier("code-796", "label-38");
        assert_eq!(item.status, BatchedQueryTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_batched_query_tier("code-796", "label-38");
        if item.move_to(BatchedQueryTierStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedQueryTierStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_query_tier("code-796", "label-38");
        item.status = BatchedQueryTierStatus::Stopped;
        assert!(
            item.move_to(BatchedQueryTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
