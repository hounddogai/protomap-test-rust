use super::*;

use acme_kit::text;

/// Where a batched segment quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSegmentQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedSegmentQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedSegmentQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched segment quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentQuota {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub ceiling: f64,
    pub description: String,
    pub checksum: String,
    pub status: BatchedSegmentQuotaStatus,
}

impl BatchedSegmentQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSegmentQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSegmentQuotaStatus) -> bool {
        match self.status {
            BatchedSegmentQuotaStatus::Created => {
                return next == BatchedSegmentQuotaStatus::Published || next == BatchedSegmentQuotaStatus::Retired;
            }
            BatchedSegmentQuotaStatus::Validated => {
                return next == BatchedSegmentQuotaStatus::Retired;
            }
            BatchedSegmentQuotaStatus::Published => {
                return next == BatchedSegmentQuotaStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSegmentQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched segment quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched segment quota", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.ceiling - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSegmentQuota {
        BatchedSegmentQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            ceiling: self.ceiling,
            description: self.description.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched segment quota in status created.
pub fn new_batched_segment_quota(code: &str, title: &str) -> BatchedSegmentQuota {
    BatchedSegmentQuota {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        ceiling: 0.0,
        description: "".to_string(),
        checksum: "".to_string(),
        status: BatchedSegmentQuotaStatus::Created,
    }
}
