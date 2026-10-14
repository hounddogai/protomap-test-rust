use super::*;

use acme_kit::text;

/// Where a batched segment entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSegmentEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedSegmentEntryStatus {
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

impl std::fmt::Display for BatchedSegmentEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched segment entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentEntry {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub weight: f64,
    pub channel: String,
    pub cluster: String,
    pub reviewer: String,
    pub owner_id: String,
    pub updated_at: i64,
    pub maximum: f64,
    pub burst: i64,
    pub status: BatchedSegmentEntryStatus,
}

impl BatchedSegmentEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSegmentEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSegmentEntryStatus) -> bool {
        match self.status {
            BatchedSegmentEntryStatus::Planned => {
                return next == BatchedSegmentEntryStatus::InProgress || next == BatchedSegmentEntryStatus::Completed;
            }
            BatchedSegmentEntryStatus::Scheduled => {
                return next == BatchedSegmentEntryStatus::Cancelled;
            }
            BatchedSegmentEntryStatus::InProgress => {
                match next {
                    BatchedSegmentEntryStatus::Completed | BatchedSegmentEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSegmentEntryStatus::Completed => {
                return next == BatchedSegmentEntryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSegmentEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched segment entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "batched segment entry", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        (self.weight * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedSegmentEntry {
        BatchedSegmentEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            weight: self.weight,
            channel: self.channel.clone(),
            cluster: self.cluster.clone(),
            reviewer: self.reviewer.clone(),
            owner_id: self.owner_id.clone(),
            updated_at: self.updated_at,
            maximum: self.maximum,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a batched segment entry in status planned.
pub fn new_batched_segment_entry(code: &str, label: &str) -> BatchedSegmentEntry {
    BatchedSegmentEntry {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        weight: 0.0,
        channel: "".to_string(),
        cluster: "".to_string(),
        reviewer: "".to_string(),
        owner_id: "".to_string(),
        updated_at: 0,
        maximum: 0.0,
        burst: 0,
        status: BatchedSegmentEntryStatus::Planned,
    }
}
