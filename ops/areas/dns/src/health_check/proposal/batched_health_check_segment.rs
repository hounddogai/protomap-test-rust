use super::*;

use acme_kit::text;

/// Where a batched health check segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedHealthCheckSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedHealthCheckSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for BatchedHealthCheckSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched health check segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckSegment {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub attempts: i64,
    pub tags: Vec<String>,
    pub checksum: String,
    pub host: String,
    pub expires_at: i64,
    pub retries: i64,
    pub status: BatchedHealthCheckSegmentStatus,
}

impl BatchedHealthCheckSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedHealthCheckSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedHealthCheckSegmentStatus) -> bool {
        match self.status {
            BatchedHealthCheckSegmentStatus::New => {
                return next == BatchedHealthCheckSegmentStatus::Running
                    || next == BatchedHealthCheckSegmentStatus::Done;
            }
            BatchedHealthCheckSegmentStatus::Queued => {
                match next {
                    BatchedHealthCheckSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedHealthCheckSegmentStatus::Running => {
                match next {
                    BatchedHealthCheckSegmentStatus::Done | BatchedHealthCheckSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedHealthCheckSegmentStatus::Done => {
                return next == BatchedHealthCheckSegmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedHealthCheckSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched health check segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched health check segment", self.code)
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
        ((self.ends_at as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedHealthCheckSegment {
        BatchedHealthCheckSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            attempts: self.attempts,
            tags: self.tags.clone(),
            checksum: self.checksum.clone(),
            host: self.host.clone(),
            expires_at: self.expires_at,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched health check segment in status new.
pub fn new_batched_health_check_segment(code: &str, label: &str) -> BatchedHealthCheckSegment {
    BatchedHealthCheckSegment {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        attempts: 0,
        tags: Vec::new(),
        checksum: "".to_string(),
        host: "".to_string(),
        expires_at: 0,
        retries: 0,
        status: BatchedHealthCheckSegmentStatus::New,
    }
}
