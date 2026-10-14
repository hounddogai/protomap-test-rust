use super::*;

use acme_kit::text;

/// Where a timeline checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineCheckpointStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TimelineCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TimelineCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineCheckpoint {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub segment: String,
    pub batch_size: i64,
    pub namespace: String,
    pub retries: i64,
    pub quota_units: i64,
    pub cluster: String,
    pub status: TimelineCheckpointStatus,
}

impl TimelineCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineCheckpointStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineCheckpointStatus) -> bool {
        match self.status {
            TimelineCheckpointStatus::Proposed => {
                match next {
                    TimelineCheckpointStatus::Rejected | TimelineCheckpointStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineCheckpointStatus::Approved => {
                match next {
                    TimelineCheckpointStatus::Rejected | TimelineCheckpointStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineCheckpointStatus::Rejected => {
                match next {
                    TimelineCheckpointStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TimelineCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.retries as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineCheckpoint {
        TimelineCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            segment: self.segment.clone(),
            batch_size: self.batch_size,
            namespace: self.namespace.clone(),
            retries: self.retries,
            quota_units: self.quota_units,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a timeline checkpoint in status proposed.
pub fn new_timeline_checkpoint(code: &str, title: &str) -> TimelineCheckpoint {
    TimelineCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        segment: "".to_string(),
        batch_size: 0,
        namespace: "".to_string(),
        retries: 0,
        quota_units: 0,
        cluster: "".to_string(),
        status: TimelineCheckpointStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_timeline_checkpoint("code-796", "title-509");
        assert_eq!(item.status, TimelineCheckpointStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_timeline_checkpoint("code-796", "title-509");
        if item.move_to(TimelineCheckpointStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineCheckpointStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_timeline_checkpoint("code-796", "title-509");
        item.status = TimelineCheckpointStatus::Withdrawn;
        assert!(
            item.move_to(TimelineCheckpointStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
