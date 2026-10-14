use super::*;

use acme_kit::text;

/// Where a seasonal freeze checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFreezeCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalFreezeCheckpointStatus {
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

impl std::fmt::Display for SeasonalFreezeCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal freeze checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeCheckpoint {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub expires_at: i64,
    pub visible: bool,
    pub ratio: f64,
    pub status: SeasonalFreezeCheckpointStatus,
}

impl SeasonalFreezeCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFreezeCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFreezeCheckpointStatus) -> bool {
        match self.status {
            SeasonalFreezeCheckpointStatus::New => {
                match next {
                    SeasonalFreezeCheckpointStatus::Running | SeasonalFreezeCheckpointStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFreezeCheckpointStatus::Queued => {
                return next == SeasonalFreezeCheckpointStatus::Running
                    || next == SeasonalFreezeCheckpointStatus::Failed;
            }
            SeasonalFreezeCheckpointStatus::Running => {
                return next == SeasonalFreezeCheckpointStatus::Failed;
            }
            SeasonalFreezeCheckpointStatus::Done => {
                match next {
                    SeasonalFreezeCheckpointStatus::Failed => {
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

    pub fn move_to(&mut self, next: SeasonalFreezeCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal freeze checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.expires_at as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalFreezeCheckpoint {
        SeasonalFreezeCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            expires_at: self.expires_at,
            visible: self.visible,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal freeze checkpoint in status new.
pub fn new_seasonal_freeze_checkpoint(code: &str, label: &str) -> SeasonalFreezeCheckpoint {
    SeasonalFreezeCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        expires_at: 0,
        visible: false,
        ratio: 0.0,
        status: SeasonalFreezeCheckpointStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_seasonal_freeze_checkpoint("code-13", "label-548");
        assert_eq!(item.status, SeasonalFreezeCheckpointStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_seasonal_freeze_checkpoint("code-13", "label-548");
        if item.move_to(SeasonalFreezeCheckpointStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalFreezeCheckpointStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_seasonal_freeze_checkpoint("code-13", "label-548");
        item.status = SeasonalFreezeCheckpointStatus::Failed;
        assert!(
            item.move_to(SeasonalFreezeCheckpointStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
