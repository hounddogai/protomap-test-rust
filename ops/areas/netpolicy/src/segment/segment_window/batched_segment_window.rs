use super::*;

use acme_kit::text;

/// Where a batched segment window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSegmentWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedSegmentWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BatchedSegmentWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched segment window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentWindow {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub revision: i64,
    pub ttl_seconds: i64,
    pub retries: i64,
    pub minimum: f64,
    pub external_ref: String,
    pub summary: String,
    pub floor_value: f64,
    pub status: BatchedSegmentWindowStatus,
}

impl BatchedSegmentWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSegmentWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSegmentWindowStatus) -> bool {
        match self.status {
            BatchedSegmentWindowStatus::Draft => {
                match next {
                    BatchedSegmentWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSegmentWindowStatus::Pending => {
                match next {
                    BatchedSegmentWindowStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSegmentWindowStatus::Active => {
                match next {
                    BatchedSegmentWindowStatus::Suspended | BatchedSegmentWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSegmentWindowStatus::Suspended => {
                return next == BatchedSegmentWindowStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSegmentWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched segment window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.floor_value - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSegmentWindow {
        BatchedSegmentWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            revision: self.revision,
            ttl_seconds: self.ttl_seconds,
            retries: self.retries,
            minimum: self.minimum,
            external_ref: self.external_ref.clone(),
            summary: self.summary.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched segment window in status draft.
pub fn new_batched_segment_window(code: &str, name: &str) -> BatchedSegmentWindow {
    BatchedSegmentWindow {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        revision: 0,
        ttl_seconds: 0,
        retries: 0,
        minimum: 0.0,
        external_ref: "".to_string(),
        summary: "".to_string(),
        floor_value: 0.0,
        status: BatchedSegmentWindowStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_segment_window("code-354", "name-162");
        assert_eq!(item.status, BatchedSegmentWindowStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_batched_segment_window("code-354", "name-162");
        if item.move_to(BatchedSegmentWindowStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSegmentWindowStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_segment_window("code-354", "name-162");
        item.status = BatchedSegmentWindowStatus::Closed;
        assert!(
            item.move_to(BatchedSegmentWindowStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
