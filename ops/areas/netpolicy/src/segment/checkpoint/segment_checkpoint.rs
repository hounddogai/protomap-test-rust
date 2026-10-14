use super::*;

use acme_kit::text;

/// Where a segment checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SegmentCheckpointStatus {
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

impl std::fmt::Display for SegmentCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a segment checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCheckpoint {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub margin: f64,
    pub summary: String,
    pub attempts: i64,
    pub namespace: String,
    pub segment: String,
    pub ends_at: i64,
    pub phone: String,
    pub status: SegmentCheckpointStatus,
}

impl SegmentCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentCheckpointStatus) -> bool {
        match self.status {
            SegmentCheckpointStatus::New => {
                return next == SegmentCheckpointStatus::Running;
            }
            SegmentCheckpointStatus::Queued => {
                return next == SegmentCheckpointStatus::Running;
            }
            SegmentCheckpointStatus::Running => {
                return next == SegmentCheckpointStatus::Done;
            }
            SegmentCheckpointStatus::Done => {
                match next {
                    SegmentCheckpointStatus::Failed => {
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

    pub fn move_to(&mut self, next: SegmentCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment checkpoint {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ends_at as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentCheckpoint {
        SegmentCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            margin: self.margin,
            summary: self.summary.clone(),
            attempts: self.attempts,
            namespace: self.namespace.clone(),
            segment: self.segment.clone(),
            ends_at: self.ends_at,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a segment checkpoint in status new.
pub fn new_segment_checkpoint(code: &str, name: &str) -> SegmentCheckpoint {
    SegmentCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        margin: 0.0,
        summary: "".to_string(),
        attempts: 0,
        namespace: "".to_string(),
        segment: "".to_string(),
        ends_at: 0,
        phone: "".to_string(),
        status: SegmentCheckpointStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_segment_checkpoint("code-407", "name-105");
        assert_eq!(item.status, SegmentCheckpointStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_segment_checkpoint("code-407", "name-105");
        if item.move_to(SegmentCheckpointStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentCheckpointStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_segment_checkpoint("code-407", "name-105");
        item.status = SegmentCheckpointStatus::Failed;
        assert!(
            item.move_to(SegmentCheckpointStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
