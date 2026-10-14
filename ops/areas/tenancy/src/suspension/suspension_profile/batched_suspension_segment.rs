use super::*;

use acme_kit::text;

/// Where a batched suspension segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSuspensionSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedSuspensionSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedSuspensionSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched suspension segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSuspensionSegment {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub checksum: String,
    pub ceiling: f64,
    pub status: BatchedSuspensionSegmentStatus,
}

impl BatchedSuspensionSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSuspensionSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSuspensionSegmentStatus) -> bool {
        match self.status {
            BatchedSuspensionSegmentStatus::Requested => {
                match next {
                    BatchedSuspensionSegmentStatus::Granted | BatchedSuspensionSegmentStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSuspensionSegmentStatus::Granted => {
                match next {
                    BatchedSuspensionSegmentStatus::Revoked | BatchedSuspensionSegmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSuspensionSegmentStatus::Revoked => {
                match next {
                    BatchedSuspensionSegmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: BatchedSuspensionSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched suspension segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> BatchedSuspensionSegment {
        BatchedSuspensionSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            checksum: self.checksum.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched suspension segment in status requested.
pub fn new_batched_suspension_segment(code: &str, label: &str) -> BatchedSuspensionSegment {
    BatchedSuspensionSegment {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        checksum: "".to_string(),
        ceiling: 0.0,
        status: BatchedSuspensionSegmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_suspension_segment("code-142", "label-140");
        assert_eq!(item.status, BatchedSuspensionSegmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_batched_suspension_segment("code-142", "label-140");
        if item.move_to(BatchedSuspensionSegmentStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSuspensionSegmentStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_suspension_segment("code-142", "label-140");
        item.status = BatchedSuspensionSegmentStatus::Expired;
        assert!(
            item.move_to(BatchedSuspensionSegmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
