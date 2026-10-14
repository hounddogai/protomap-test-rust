use super::*;

use acme_kit::text;

/// Where a postmortem slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PostmortemSlotStatus {
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

impl std::fmt::Display for PostmortemSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemSlot {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub created_at: i64,
    pub version: i64,
    pub namespace: String,
    pub source: String,
    pub expires_at: i64,
    pub status: PostmortemSlotStatus,
}

impl PostmortemSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemSlotStatus) -> bool {
        match self.status {
            PostmortemSlotStatus::New => {
                return next == PostmortemSlotStatus::Queued || next == PostmortemSlotStatus::Done;
            }
            PostmortemSlotStatus::Queued => {
                return next == PostmortemSlotStatus::Done || next == PostmortemSlotStatus::Failed;
            }
            PostmortemSlotStatus::Running => {
                return next == PostmortemSlotStatus::Done || next == PostmortemSlotStatus::Failed;
            }
            PostmortemSlotStatus::Done => {
                match next {
                    PostmortemSlotStatus::Failed => {
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

    pub fn move_to(&mut self, next: PostmortemSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "postmortem slot", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.created_at as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemSlot {
        PostmortemSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            created_at: self.created_at,
            version: self.version,
            namespace: self.namespace.clone(),
            source: self.source.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a postmortem slot in status new.
pub fn new_postmortem_slot(code: &str, title: &str) -> PostmortemSlot {
    PostmortemSlot {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        created_at: 0,
        version: 0,
        namespace: "".to_string(),
        source: "".to_string(),
        expires_at: 0,
        status: PostmortemSlotStatus::New,
    }
}
