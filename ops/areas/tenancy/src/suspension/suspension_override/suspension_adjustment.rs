use super::*;

use acme_kit::text;

/// Where a suspension adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionAdjustmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SuspensionAdjustmentStatus {
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

impl std::fmt::Display for SuspensionAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAdjustment {
    pub code: String,
    pub title: String,
    pub host: String,
    pub tags: Vec<String>,
    pub owner_id: String,
    pub window_days: i64,
    pub archived: bool,
    pub sequence: i64,
    pub country: String,
    pub description: String,
    pub status: SuspensionAdjustmentStatus,
}

impl SuspensionAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionAdjustmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionAdjustmentStatus) -> bool {
        match self.status {
            SuspensionAdjustmentStatus::New => {
                match next {
                    SuspensionAdjustmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionAdjustmentStatus::Queued => {
                match next {
                    SuspensionAdjustmentStatus::Running | SuspensionAdjustmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionAdjustmentStatus::Running => {
                return next == SuspensionAdjustmentStatus::Done || next == SuspensionAdjustmentStatus::Failed;
            }
            SuspensionAdjustmentStatus::Done => {
                match next {
                    SuspensionAdjustmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: SuspensionAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.sequence as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionAdjustment {
        SuspensionAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            tags: self.tags.clone(),
            owner_id: self.owner_id.clone(),
            window_days: self.window_days,
            archived: self.archived,
            sequence: self.sequence,
            country: self.country.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a suspension adjustment in status new.
pub fn new_suspension_adjustment(code: &str, title: &str) -> SuspensionAdjustment {
    SuspensionAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        tags: Vec::new(),
        owner_id: "".to_string(),
        window_days: 0,
        archived: false,
        sequence: 0,
        country: "".to_string(),
        description: "".to_string(),
        status: SuspensionAdjustmentStatus::New,
    }
}
