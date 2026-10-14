use super::*;

use acme_kit::text;

/// Where a batched policy segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPolicySegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedPolicySegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for BatchedPolicySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched policy segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicySegment {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub revision: i64,
    pub source: String,
    pub threshold: f64,
    pub ceiling: f64,
    pub status: BatchedPolicySegmentStatus,
}

impl BatchedPolicySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPolicySegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPolicySegmentStatus) -> bool {
        match self.status {
            BatchedPolicySegmentStatus::Planned => {
                return next == BatchedPolicySegmentStatus::Completed || next == BatchedPolicySegmentStatus::Cancelled;
            }
            BatchedPolicySegmentStatus::Scheduled => {
                match next {
                    BatchedPolicySegmentStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedPolicySegmentStatus::InProgress => {
                return next == BatchedPolicySegmentStatus::Completed;
            }
            BatchedPolicySegmentStatus::Completed => {
                match next {
                    BatchedPolicySegmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BatchedPolicySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched policy segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched policy segment", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ceiling + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPolicySegment {
        BatchedPolicySegment {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            revision: self.revision,
            source: self.source.clone(),
            threshold: self.threshold,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a batched policy segment in status planned.
pub fn new_batched_policy_segment(code: &str, title: &str) -> BatchedPolicySegment {
    BatchedPolicySegment {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        revision: 0,
        source: "".to_string(),
        threshold: 0.0,
        ceiling: 0.0,
        status: BatchedPolicySegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_policy_segment("code-235", "title-474");
        assert_eq!(item.status, BatchedPolicySegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_batched_policy_segment("code-235", "title-474");
        if item.move_to(BatchedPolicySegmentStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPolicySegmentStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_policy_segment("code-235", "title-474");
        item.status = BatchedPolicySegmentStatus::Cancelled;
        assert!(
            item.move_to(BatchedPolicySegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
