use super::*;

use acme_kit::text;

/// Where a default migration variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultMigrationVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultMigrationVarianceStatus {
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

impl std::fmt::Display for DefaultMigrationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default migration variance as the migration team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationVariance {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub archived: bool,
    pub visible: bool,
    pub notes: String,
    pub segment: String,
    pub floor_value: f64,
    pub status: DefaultMigrationVarianceStatus,
}

impl DefaultMigrationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultMigrationVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultMigrationVarianceStatus) -> bool {
        match self.status {
            DefaultMigrationVarianceStatus::New => {
                match next {
                    DefaultMigrationVarianceStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMigrationVarianceStatus::Queued => {
                match next {
                    DefaultMigrationVarianceStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMigrationVarianceStatus::Running => {
                return next == DefaultMigrationVarianceStatus::Done || next == DefaultMigrationVarianceStatus::Failed;
            }
            DefaultMigrationVarianceStatus::Done => {
                return next == DefaultMigrationVarianceStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultMigrationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default migration variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default migration variance", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.floor_value - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultMigrationVariance {
        DefaultMigrationVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            archived: self.archived,
            visible: self.visible,
            notes: self.notes.clone(),
            segment: self.segment.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a default migration variance in status new.
pub fn new_default_migration_variance(code: &str, title: &str) -> DefaultMigrationVariance {
    DefaultMigrationVariance {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        archived: false,
        visible: false,
        notes: "".to_string(),
        segment: "".to_string(),
        floor_value: 0.0,
        status: DefaultMigrationVarianceStatus::New,
    }
}
