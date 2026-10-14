use super::*;

use acme_kit::text;

/// Where a batched sink segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSinkSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedSinkSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedSinkSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched sink segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkSegment {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub threshold: f64,
    pub reviewer: String,
    pub expires_at: i64,
    pub external_ref: String,
    pub rate: f64,
    pub version: i64,
    pub score: f64,
    pub weight: f64,
    pub status: BatchedSinkSegmentStatus,
}

impl BatchedSinkSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSinkSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSinkSegmentStatus) -> bool {
        match self.status {
            BatchedSinkSegmentStatus::Created => {
                match next {
                    BatchedSinkSegmentStatus::Validated | BatchedSinkSegmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSinkSegmentStatus::Validated => {
                return next == BatchedSinkSegmentStatus::Published;
            }
            BatchedSinkSegmentStatus::Published => {
                match next {
                    BatchedSinkSegmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: BatchedSinkSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched sink segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.priority as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSinkSegment {
        BatchedSinkSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            threshold: self.threshold,
            reviewer: self.reviewer.clone(),
            expires_at: self.expires_at,
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            version: self.version,
            score: self.score,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched sink segment in status created.
pub fn new_batched_sink_segment(code: &str, label: &str) -> BatchedSinkSegment {
    BatchedSinkSegment {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        threshold: 0.0,
        reviewer: "".to_string(),
        expires_at: 0,
        external_ref: "".to_string(),
        rate: 0.0,
        version: 0,
        score: 0.0,
        weight: 0.0,
        status: BatchedSinkSegmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_sink_segment("code-429", "label-879");
        assert_eq!(item.status, BatchedSinkSegmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_batched_sink_segment("code-429", "label-879");
        if item.move_to(BatchedSinkSegmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSinkSegmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_sink_segment("code-429", "label-879");
        item.status = BatchedSinkSegmentStatus::Retired;
        assert!(
            item.move_to(BatchedSinkSegmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
