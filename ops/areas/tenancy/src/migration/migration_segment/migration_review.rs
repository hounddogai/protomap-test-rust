use super::*;

use acme_kit::text;

/// Where a migration review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MigrationReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MigrationReviewStatus {
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

impl std::fmt::Display for MigrationReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One migration review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationReview {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub channel: String,
    pub summary: String,
    pub audience: String,
    pub locale: String,
    pub status: MigrationReviewStatus,
}

impl MigrationReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MigrationReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MigrationReviewStatus) -> bool {
        match self.status {
            MigrationReviewStatus::New => {
                return next == MigrationReviewStatus::Running || next == MigrationReviewStatus::Failed;
            }
            MigrationReviewStatus::Queued => {
                return next == MigrationReviewStatus::Done;
            }
            MigrationReviewStatus::Running => {
                return next == MigrationReviewStatus::Done || next == MigrationReviewStatus::Failed;
            }
            MigrationReviewStatus::Done => {
                match next {
                    MigrationReviewStatus::Failed => {
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

    pub fn move_to(&mut self, next: MigrationReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "migration review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.sequence as f64
    }

    pub fn copy(&self) -> MigrationReview {
        MigrationReview {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            channel: self.channel.clone(),
            summary: self.summary.clone(),
            audience: self.audience.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a migration review in status new.
pub fn new_migration_review(code: &str, label: &str) -> MigrationReview {
    MigrationReview {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        channel: "".to_string(),
        summary: "".to_string(),
        audience: "".to_string(),
        locale: "".to_string(),
        status: MigrationReviewStatus::New,
    }
}
