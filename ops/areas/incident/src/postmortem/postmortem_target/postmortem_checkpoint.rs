use super::*;

use acme_kit::text;

/// Where a postmortem checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PostmortemCheckpointStatus {
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

impl std::fmt::Display for PostmortemCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCheckpoint {
    pub code: String,
    pub name: String,
    pub description: String,
    pub port: i64,
    pub segment: String,
    pub status: PostmortemCheckpointStatus,
}

impl PostmortemCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemCheckpointStatus) -> bool {
        match self.status {
            PostmortemCheckpointStatus::New => {
                return next == PostmortemCheckpointStatus::Done || next == PostmortemCheckpointStatus::Failed;
            }
            PostmortemCheckpointStatus::Queued => {
                match next {
                    PostmortemCheckpointStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemCheckpointStatus::Running => {
                match next {
                    PostmortemCheckpointStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemCheckpointStatus::Done => {
                return next == PostmortemCheckpointStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> PostmortemCheckpoint {
        PostmortemCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            port: self.port,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a postmortem checkpoint in status new.
pub fn new_postmortem_checkpoint(code: &str, name: &str) -> PostmortemCheckpoint {
    PostmortemCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        port: 0,
        segment: "".to_string(),
        status: PostmortemCheckpointStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_postmortem_checkpoint("code-387", "name-515");
        assert_eq!(item.status, PostmortemCheckpointStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_postmortem_checkpoint("code-387", "name-515");
        if item.move_to(PostmortemCheckpointStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemCheckpointStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_postmortem_checkpoint("code-387", "name-515");
        item.status = PostmortemCheckpointStatus::Failed;
        assert!(
            item.move_to(PostmortemCheckpointStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
