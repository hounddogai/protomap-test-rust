use super::*;

use acme_kit::text;

/// Where a peering checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PeeringCheckpointStatus {
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

impl std::fmt::Display for PeeringCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringCheckpoint {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub label: String,
    pub floor_value: f64,
    pub key_id: String,
    pub host: String,
    pub country: String,
    pub grace_days: i64,
    pub cipher: String,
    pub status: PeeringCheckpointStatus,
}

impl PeeringCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringCheckpointStatus) -> bool {
        match self.status {
            PeeringCheckpointStatus::New => {
                match next {
                    PeeringCheckpointStatus::Done | PeeringCheckpointStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringCheckpointStatus::Queued => {
                match next {
                    PeeringCheckpointStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringCheckpointStatus::Running => {
                match next {
                    PeeringCheckpointStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringCheckpointStatus::Done => {
                return next == PeeringCheckpointStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.grace_days as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringCheckpoint {
        PeeringCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            label: self.label.clone(),
            floor_value: self.floor_value,
            key_id: self.key_id.clone(),
            host: self.host.clone(),
            country: self.country.clone(),
            grace_days: self.grace_days,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a peering checkpoint in status new.
pub fn new_peering_checkpoint(code: &str, title: &str) -> PeeringCheckpoint {
    PeeringCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        label: "".to_string(),
        floor_value: 0.0,
        key_id: "".to_string(),
        host: "".to_string(),
        country: "".to_string(),
        grace_days: 0,
        cipher: "".to_string(),
        status: PeeringCheckpointStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_peering_checkpoint("code-263", "title-849");
        assert_eq!(item.status, PeeringCheckpointStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_peering_checkpoint("code-263", "title-849");
        if item.move_to(PeeringCheckpointStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringCheckpointStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_peering_checkpoint("code-263", "title-849");
        item.status = PeeringCheckpointStatus::Failed;
        assert!(
            item.move_to(PeeringCheckpointStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
